#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <stdint.h>
#include <stdlib.h>

#include "sensor.h"






int main()
{
    supported_sensor_init();
    get_sensor_list();

    sensor_dev_t *bmi320 = (sensor_dev_t *)malloc(sizeof(sensor_dev_t) * 1);
    bmi320->idx = 0;
    bmi320->dev_name = "bmi320";
    bmi320->vendor_name = "bosch";
    sensor_dev_register(ACC,  bmi320);
    get_sensor_list();

    listener_t *l1 = (listener_t *)malloc(sizeof(listener_t) * 1);
    l1->req_odr = 120;
    sensor_listener_register(ACC, 0, l1);

    listener_t *l2 = (listener_t *)malloc(sizeof(listener_t) * 1);
    l2->req_odr = 200;
    sensor_listener_register(ACC, 0, l2);

    listener_t *l3 = (listener_t *)malloc(sizeof(listener_t) * 1);
    l3->req_odr = 400;
    sensor_listener_register(ACC, 0, l3);

    get_sensor_list();
#if 0
    sensor_listener_unregister(l2);
    get_sensor_list();

    sensor_listener_unregister(l3);
    get_sensor_list();
#endif
    sensor_dev_unregister(bmi320);
    get_sensor_list();


    return 0;
}
