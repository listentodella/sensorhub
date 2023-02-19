#ifndef __SENSOR_H_
#define __SENSOR_H_
#include "list.h"
#include "notifier.h"
#include <stdint.h>

#define SENSOR_HZ(_hz)  ((uint32_t)(_hz))
//#define ARRAY_SIZE(arr) (sizeof(arr)/sizeof((arr)[0]) + __must_be_array(arr))
#define ARRAY_SIZE(arr) (sizeof(arr)/sizeof((arr)[0]))

typedef enum sensor_type {
    ACC,
    GYR,
    MAG,
    TEMP,
    ALG0,
    ALG1,
    UNKNOWN_TYPE
} sensor_type_t;

//assume only 3 axis
typedef struct axis_map {
    uint8_t axis[3];//x,y,z

    //each axis represents as an integer in range[-3, 3], excluding 0
    //1 -> x, 2 -> y, 3 -> z
    //-1 -> -x, -2->-y, -3->-z
    //so if want to map the chip's X to device's -Y, set sign[0] = -2
    int8_t  sign[3];
} axis_map_t;


/*
 * sensor_list
 *  |------> ACC
 *  |        |---> acc_dev0
 *  |        |      |------> listener0 -> listener1 -> ...
 *  |        |      |------> odr0 -> odr1 -> ...
 *  |        |---> acc_dev1
 *  |        |      |------> listener0 -> listener1 -> ...
 *  |        |      |------> odr0 -> odr1 -> ...
 *  |        |---> acc_dev...
 *  |                |------> listener0 -> listener1 -> ...
 *  |                |------> odr0 -> odr1 -> ...
 *  |------> GYR
 *  |         |---> gyr_dev0
 *  |         |      |------> listener0 -> listener1 -> ...
 *  |         |      |------> odr0 -> odr1 -> ...
 *  |         |---> gyr_dev...
 *  |                |------> listener0 -> listener1 -> ...
 *  |                |------> odr0 -> odr1 -> ...
 *
 * */

typedef struct sensor {
    char name[32];
    void *priv_data;
    sensor_type_t type;
    struct list_head dev_list;
    struct list_head node;
} sensor_t;

typedef struct sensor_dev {
    uint8_t idx;
    char *dev_name;
    char *vendor_name;
    axis_map_t axis_map;
    struct list_head listener_list;
    struct list_head odr_list;
    struct list_head node;
} sensor_dev_t;

typedef struct listener {
    uint32_t req_odr;
    uint32_t matched_odr;
    struct notifier_block nb;
    struct list_head node;
} listener_t;

typedef struct odr {
    uint8_t ref_cnt;
    uint32_t rate;
    struct list_head node;
} odr_t;


void get_sensor_list();
void supported_sensor_init();
//int sensor_dev_register(sensor_type_t type, uint8_t idx, sensor_dev_t *dev);
int sensor_dev_register(sensor_type_t type, sensor_dev_t *dev);
void sensor_dev_unregister(sensor_dev_t *dev);


int sensor_listener_register(sensor_type_t type, uint8_t idx, listener_t *listener);
void sensor_listener_unregister(listener_t *listener);











void sensor_axis_map_init(axis_map_t *axis_map, uint8_t *axis_cfg);
void sensor_data_map_axis(axis_map_t *axis_map, int16_t *data);

#endif
