#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>


#include "list.h"
#include "sensor.h"


static int sensor_odr_unregister(sensor_type_t type, uint8_t idx, uint32_t matched_odr);


static LIST_HEAD(sensor_list);
//static LIST_HEAD(sensor_dev_list);
//static LIST_HEAD(listener_list);
//static LIST_HEAD(odr_list);

static sensor_t supported_sensors[] = {
    { .name = "ACC", .type = ACC },
    { .name = "GYR", .type = GYR },
    { .name = "MAG", .type = MAG },
    { .name = "TEMP", .type = TEMP },
    { .name = "ALG0", .type = ALG0 },
    { .name = "ALG1", .type = ALG1 },
};



static int sensor_type_register(sensor_t *sensor)
{
    int ret = 0;

    INIT_LIST_HEAD(&sensor->dev_list);
    list_add_tail(&sensor->node, &sensor_list);

    return ret;
}


static int sensor_type_unregister(sensor_t *sensor)
{
    int ret = 0;
    //before we remove this node, please check whether need disconnect something else

    //we just need remove this node
    list_del_init(&sensor->node);

    return ret;
}


void get_sensor_list()
{
    sensor_t *sensor, *tmp;
    sensor_dev_t *dev, *dev_tmp;
    listener_t *l, *ltmp;
    odr_t *o, *otmp;
    list_for_each_entry_safe(sensor, tmp, &sensor_list, node) {
        printf("get sensor %d, name %s\n", sensor->type, sensor->name);
        list_for_each_entry_safe(dev, dev_tmp, &sensor->dev_list, node) {
            printf("\tidx = %d, dev name %s, vendor name = %s\n",
                    dev->idx, dev->dev_name, dev->vendor_name);
            list_for_each_entry_safe(l, ltmp, &dev->listener_list, node) {
                printf("\tlistener->req_odr = %d, matched_odr = %d\n",
                        l->req_odr, l->matched_odr);
            }
            list_for_each_entry_safe(o, otmp, &dev->odr_list, node) {
                printf("\todr->ref_cnt = %d, rate = %d\n",
                        o->ref_cnt, o->rate);
            }
        }
    }

    printf("========================\n\n\n");
}


void supported_sensor_init()
{
    for (int i = 0; i < ARRAY_SIZE(supported_sensors); i++) {
        sensor_type_register(&supported_sensors[i]);
    }
}


sensor_dev_t *sensor_allocate_dev(void)
{
    sensor_dev_t *dev = (sensor_dev_t *)calloc(1, sizeof(sensor_dev_t));
    if (!dev) {
        printf("%s:failed to allocate device!\n", __func__);
        goto out;
    }
    /* fill members if need */
    dev->idx = 0;
    INIT_LIST_HEAD(&dev->listener_list);
    INIT_LIST_HEAD(&dev->odr_list);
    INIT_LIST_HEAD(&dev->node);

out:
    return dev;
}


//int sensor_dev_register(sensor_type_t type, uint8_t idx, sensor_dev_t *dev)
int sensor_dev_register(sensor_type_t type, sensor_dev_t *dev)
{
    int ret = -1;
    sensor_t *sensor, *tmp;
    sensor_dev_t *sensor_dev, *dtmp;

    list_for_each_entry_safe(sensor, tmp, &sensor_list, node) {
        if (sensor->type == type) {
            list_for_each_entry_safe(sensor_dev, dtmp, &sensor->dev_list, node) {
                if (sensor_dev->idx == dev->idx) {
                    printf("%s:repeated idx %d for type %d, please check!\n", __func__, dev->idx, type);
                    goto out;
                }
            }

            ret = 0;
            list_add_tail(&dev->node, &sensor->dev_list);
        }
    }

out:
    return ret;
}

//int sensor_dev_unregister(sensor_type_t type, uint8_t idx, sensor_dev_t *dev)
void sensor_dev_unregister(sensor_dev_t *dev)
{
    //TODO:please make sure all resources released such as listener,odr
    printf("%s:should release all listeners & odrs!\n", __func__);
    uint8_t dflag = 0;
    sensor_t *sensor, *tmp;
    sensor_dev_t *d, *dtmp;
    listener_t *l, *ltmp;
    odr_t *o, *otmp;
    list_for_each_entry_safe(sensor, tmp, &sensor_list, node) {
        list_for_each_entry_safe(d, dtmp, &sensor->dev_list, node) {
            if (dev == d) {
                dflag = 1;
                /* Listeners can only request dev through listener_register,
                 * so dev's odr_list is totally fulled by listener_register.
                 * Thus, through listener_unregister, all odrs will be unregistered
                 */
                list_for_each_entry_safe(l, ltmp, &dev->listener_list, node) {
                    sensor_listener_unregister(l);
                }
                //list_for_each_entry_safe(o, otmp, &dev->odr_list, node) {
                //    sensor_odr_unregister(sensor->type, dev->idx, o->rate);
                //}
                goto out;
            }
        }
    }

out:
    if (dflag) {
        list_del_init(&dev->node);
        free(dev);
        dev = NULL;
    } else {
        printf("failed to find target sensor!\n");
    }
}

//TODO:remember to adapt different supported odrs for different sensors
#if 0
static uint8_t sensor_compute_odr(float *sample_rate)
{
    uint8_t odr = 0x00;
    uint32_t rate = (uint32_t)(*sample_rate);

    if (rate > SENSOR_HZ(1600)) {
        rate = SENSOR_HZ(3200);

    } else if (rate > SENSOR_HZ(800)) {
        rate = SENSOR_HZ(1600);

    } else if (rate > SENSOR_HZ(400)) {
        rate = SENSOR_HZ(800);

    } else if (rate > SENSOR_HZ(200)) {
        rate = SENSOR_HZ(400);

    } else if (rate > SENSOR_HZ(100)) {
        rate = SENSOR_HZ(200);

    } else if (rate > SENSOR_HZ(50)) {
        rate = SENSOR_HZ(100);

    } else if (rate > SENSOR_HZ(25)) {
        rate = SENSOR_HZ(50);

    } else if (rate > SENSOR_HZ(25.0f / 2.0f)) {
        rate = SENSOR_HZ(25);

    } else if (rate > SENSOR_HZ(25.0f / 4.0f)) {
        rate = SENSOR_HZ(25.0f / 2.0f);

    } else if (rate > SENSOR_HZ(25.0f / 8.0f)) {
        rate = SENSOR_HZ(25.0f / 4.0f);

    } else if (rate > SENSOR_HZ(25.0f / 16.0f)) {
        rate = SENSOR_HZ(25.0f / 8.0f);

    } else if (rate > SENSOR_HZ(25.0f / 32.0f)) {
        rate = SENSOR_HZ(25.0f / 16.0f);

    } else {
        rate = SENSOR_HZ(25.0f / 32.0f);

    }
    *sample_rate = rate;

    switch (rate) {
        // fall through intended to get the correct register value
        case SENSOR_HZ(6400):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(3200):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(1600):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(800):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(400):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(200):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(100):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(50):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(25):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(25.0f / 2.0f):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(25.0f / 4.0f):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(25.0f / 8.0f):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(25.0f / 16.0f):
            odr++;
            //lint -fallthrough
        case SENSOR_HZ(25.0f / 32.0f):
            odr++;
            //lint -fallthrough
        default:
            return odr;
    }
}
#endif

static uint32_t sensor_compute_odr(float req_rate)
{
    uint32_t rate = (uint32_t)(req_rate);

    if (rate > SENSOR_HZ(1600)) {
        rate = SENSOR_HZ(3200);
    } else if (rate > SENSOR_HZ(800)) {
        rate = SENSOR_HZ(1600);
    } else if (rate > SENSOR_HZ(400)) {
        rate = SENSOR_HZ(800);
    } else if (rate > SENSOR_HZ(200)) {
        rate = SENSOR_HZ(400);
    } else if (rate > SENSOR_HZ(100)) {
        rate = SENSOR_HZ(200);
    } else if (rate > SENSOR_HZ(50)) {
        rate = SENSOR_HZ(100);
    } else if (rate > SENSOR_HZ(25)) {
        rate = SENSOR_HZ(50);
    } else if (rate > SENSOR_HZ(25.0f / 2.0f)) {
        rate = SENSOR_HZ(25);
    } else if (rate > SENSOR_HZ(25.0f / 4.0f)) {
        rate = SENSOR_HZ(25.0f / 2.0f);
    } else if (rate > SENSOR_HZ(25.0f / 8.0f)) {
        rate = SENSOR_HZ(25.0f / 4.0f);
    } else if (rate > SENSOR_HZ(25.0f / 16.0f)) {
        rate = SENSOR_HZ(25.0f / 8.0f);
    } else if (rate > SENSOR_HZ(25.0f / 32.0f)) {
        rate = SENSOR_HZ(25.0f / 16.0f);
    } else {
        rate = SENSOR_HZ(25.0f / 32.0f);
    }

    return rate;
}



static int sensor_odr_register(sensor_type_t type, uint8_t idx,
                                uint32_t req_odr, uint32_t *matched_odr)
{
    int ret = 0;
    odr_t *odr = NULL;
    sensor_t *sensor = NULL, *tmp = NULL;
    sensor_dev_t *dev = NULL, *dev_tmp = NULL;
    odr_t *o = NULL, *otmp = NULL;
    uint8_t odr_flag = 0, dev_flag = 0;

    *matched_odr = sensor_compute_odr(req_odr);
    printf("req_odr = %d hz, matched_odr = %d hz\n", req_odr, *matched_odr);

    list_for_each_entry_safe(sensor, tmp, &sensor_list, node) {
        list_for_each_entry_safe(dev, dev_tmp, &sensor->dev_list, node) {
            if (sensor->type == type && dev->idx == idx) {
                dev_flag = 1;
                list_for_each_entry_safe(o, otmp, &dev->odr_list, node) {
                    if (o->rate == *matched_odr) {
                        odr_flag = 1;
                        o->ref_cnt++;
                        printf("++ ref_cnt = %d for sensor_type %d-%d, rate = %d\n",
                                o->ref_cnt, sensor->type, dev->idx, o->rate);
                        //for same rate, only one odr_list_node
                        //break;
                        goto out_find_odr;
                    }
                }
                //target sensor & dev can have only one odr_list
                //break;
                goto out_find_dev;
            }
        }
    }

out_find_dev:
    if (!dev_flag) {
        printf("target sensor %d-%d doesn't exist!\n", sensor->type, dev->idx);
        ret = -1;
        goto out;
    }

out_find_odr:
    if (!odr_flag) {
        odr = (odr_t *)malloc(sizeof(odr_t) * 1);
        odr->ref_cnt = 1;
        odr->rate    = *matched_odr;
        list_add_tail(&odr->node, &dev->odr_list);
        printf("create a new odr node for sensor %d-%d\n", sensor->type, dev->idx);
    }


out:
    return ret;
}


static int sensor_odr_unregister(sensor_type_t type, uint8_t idx, uint32_t matched_odr)
{
    int ret = 0;
    odr_t *odr = NULL;
    sensor_t *sensor = NULL, *tmp = NULL;
    sensor_dev_t *dev = NULL, *dev_tmp = NULL;
    odr_t *o = NULL, *otmp = NULL;
    uint8_t odr_flag = 0, dev_flag = 0;

    printf("%s:should decrease ref_cnt for matched_odr = %d hz\n", __func__, matched_odr);

    list_for_each_entry_safe(sensor, tmp, &sensor_list, node) {
        list_for_each_entry_safe(dev, dev_tmp, &sensor->dev_list, node) {
            if (sensor->type == type && dev->idx == idx) {
                dev_flag = 1;
                list_for_each_entry_safe(o, otmp, &dev->odr_list, node) {
                    if (o->rate == matched_odr) {
                        odr_flag = 1;
                        o->ref_cnt--;
                        printf("-- ref_cnt = %d for sensor_type %d-%d, rate = %d\n",
                                o->ref_cnt, sensor->type, dev->idx, o->rate);
                        //for same rate, only one odr_list_node
                        //break;
                        goto out_find_odr;
                    }
                }
                //target sensor & dev can have only one odr_list
                //break;
                goto out_find_dev;
            }
        }
    }

out_find_dev:
    if (!dev_flag) {
        printf("target sensor %d-%d doesn't exist!\n", sensor->type, dev->idx);
        ret = -1;
        goto out;
    }

out_find_odr:
    if (odr_flag) {
        if (!o->ref_cnt) {
            list_del_init(&o->node);
            //list_del(&o->node);
            free(o);
            o = NULL;
            printf("%s:should del odr node for sensor %d-%d\n",
                    __func__, sensor->type, dev->idx);
        }
    } else {
        ret = -2;
        printf("cannot find a matched_odr for this listener!\n");
    }

out:
    return ret;
}

listener_t *sensor_allocate_listener(void)
{
    listener_t *l = (listener_t *)calloc(1, sizeof(listener_t));
    if (!l) {
        printf("%s:failed to allocate listener!\n", __func__);
        goto out;
    }
    /* fill members if need */
    l->req_odr = 0;
    l->matched_odr = 0;

out:
    return l;
}



int sensor_listener_register(sensor_type_t type, uint8_t idx, listener_t *listener)
{
    int ret = -1;
    sensor_t *sensor, *tmp;
    sensor_dev_t *dev, *dev_tmp;

    list_for_each_entry_safe(sensor, tmp, &sensor_list, node) {
        if (sensor->type == type) {
            printf("get sensor %d, name %s\n", sensor->type, sensor->name);
            list_for_each_entry_safe(dev, dev_tmp, &sensor->dev_list, node) {
                if (dev->idx == idx) {
                    ret = 0;
                    printf("idx = %d,dev name %s, vendor name = %s\n",
                            dev->idx, dev->dev_name, dev->vendor_name);
                    list_add_tail(&listener->node, &dev->listener_list);
                    sensor_odr_register(type, idx, listener->req_odr, &listener->matched_odr);
                }
            }
        }
    }

    return ret;
}

void sensor_listener_unregister(listener_t *listener)
{
    //TODO:please make sure all resources released such as listener,odr
    printf("%s:should release such listener and responding odr requested!\n", __func__);

    sensor_t *sensor = NULL, *tmp = NULL;
    sensor_dev_t *dev = NULL, *dev_tmp = NULL;
    listener_t *l = NULL, *ltmp = NULL;
    uint8_t odr_flag = 0, dev_flag = 0, lflag = 0;

    list_for_each_entry_safe(sensor, tmp, &sensor_list, node) {
        list_for_each_entry_safe(dev, dev_tmp, &sensor->dev_list, node) {
            list_for_each_entry_safe(l, ltmp, &dev->listener_list, node) {
                if (l == listener) {
                    lflag = 1;
                    sensor_odr_unregister(sensor->type, dev->idx, l->matched_odr);
                    goto out;
                    //break;
                }
            }
        }
    }

out:
    if (lflag) {
        list_del_init(&listener->node);
        //make sure the listener is on the heap
        free(listener);
        listener = NULL;
    } else {
        printf("failed to find such listener!\n");
    }
}








