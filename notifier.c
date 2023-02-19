#include <stdio.h>
#include <string.h>
#include "notifier.h"


int notifier_chain_register(struct notifier_block **nl, struct notifier_block *n)
{
    while ((*nl) != NULL) {
        if (n->priority > (*nl)->priority) {
            break;
        }
        nl = &((*nl)->next);
    }

    n->next = *nl;

    //rcu_assign_pointer(*nl, n);
    *nl = n;

    return 0;
}


int notifier_chain_unregister(struct notifier_block **nl, struct notifier_block *n)
{
    while ((*nl) != NULL) {
        if ((*nl) == n) {
            //rcu_assign_pointer(*nl, n->next);
            *nl = n->next;
            return 0;
        }
        nl = &((*nl)->next);
    }

    //return -ENOENT;
    return -1;
}


int notifier_call_chain(struct notifier_block **nl, unsigned long val, void *v,
                        int nr_to_call, int *nr_calls)
{
    int ret = 0;
    struct notifier_block *nb, *next_nb;

    //nb = rcu_dereference(*nl);
    nb = *nl;

    while (nb && nr_to_call) {
        //next_nb = rcu_dereference(nb->next);
        next_nb = nb->next;

        ret = nb->notifier_call(nb, val, v);

        if (nr_calls) {
            (*nr_calls)++;
        }

        if ((ret & NOTIFY_STOP_MASK) == NOTIFY_STOP_MASK) {
            break;
        }

        nb = next_nb;

        nr_to_call--;

    }

    return ret;
}












