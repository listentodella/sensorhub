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

    printf("=====================register devices=====================\n");
    sensor_dev_t *bmi320 = sensor_allocate_dev();
    bmi320->idx = 0;
    bmi320->dev_name = "bmi320";
    bmi320->vendor_name = "bosch";
    sensor_dev_register(ACC,  bmi320);

    sensor_dev_t *bmi260 = sensor_allocate_dev();
    bmi260->idx = 0;
    bmi260->dev_name = "bmi260";
    bmi260->vendor_name = "bosch";
    sensor_dev_register(GYR,  bmi260);

    sensor_dev_t *bmi160 = sensor_allocate_dev();
    bmi160->idx = 0;
    bmi160->dev_name = "bmi160";
    bmi160->vendor_name = "bosch";
    sensor_dev_register(ACC,  bmi160);



    get_sensor_list();

    printf("=====================register listeners=====================\n");
    listener_t *l1 = sensor_allocate_listener();
    l1->req_odr = 120;
    sensor_listener_register(ACC, 0, l1);

    listener_t *l2 = sensor_allocate_listener();
    l2->req_odr = 200;
    sensor_listener_register(ACC, 0, l2);

    listener_t *l3 = sensor_allocate_listener();
    l3->req_odr = 400;
    sensor_listener_register(ACC, 0, l3);

    listener_t *l4 = sensor_allocate_listener();
    l4->req_odr = 800;
    sensor_listener_register(GYR, 0, l4);

    get_sensor_list();


#if 1
    printf("=====================unregister listeners=====================\n");
    sensor_listener_unregister(l2);
    //get_sensor_list();

    sensor_listener_unregister(l3);
    get_sensor_list();
#endif

    printf("=====================unregister a dev=====================\n");
    sensor_dev_unregister(bmi320);
    sensor_dev_unregister(bmi260);
    get_sensor_list();


    return 0;
}
