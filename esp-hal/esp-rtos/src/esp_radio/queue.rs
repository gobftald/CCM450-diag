use esp_radio_rtos_driver::{
    queue::{CompatQueue, QueuePtr},
    register_queue_implementation,
};

register_queue_implementation!(CompatQueue);
