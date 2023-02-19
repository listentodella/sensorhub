#ifndef __NOTIFY__
#define __NOTIFY__

#define NOTIFY_DONE 0x0000
#define NOTIFY_OK   0x0001
#define NOTIFY_STOP_MASK 0x8000
#define NOTIFY_BAD  (NOTIFY_STOP_MASK | 0x0002)


#define INIT_NOTIFIER_HEAD(name)   do { \
            (name)->head = NULL; \
        } while(0)

struct notifier_block;

typedef int (*notifier_fn_t) (struct notifier_block *nb,
                    unsigned long action, void *data);

struct notifier_block {
    int priority;
    notifier_fn_t notifier_call;
    struct notifier_block *next;
};

struct notifer_head {
    struct notifier_block *head;
};


int notifier_chain_register(struct notifier_block **nl, struct notifier_block *n);
int notifier_chain_unregister(struct notifier_block **nl, struct notifier_block *n);
int notifier_call_chain(struct notifier_block **nl, unsigned long val, void *v,
                        int nr_to_call, int *nr_calls);



#endif
