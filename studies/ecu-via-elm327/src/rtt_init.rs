#[cfg(feature = "rtt")]
use rtt_target::{rtt_init, /*rprintln,*/ ChannelMode};

#[cfg(feature = "rtos_trace")]
use systemview_target::SystemView;

#[cfg(feature = "rtos_trace")]
rtos_trace::global_trace! { SystemView }

pub(crate) fn rtt_init() {
// initializing rtt_target only
    #[cfg(all(feature = "rtt", not(feature = "rtos_trace")))]
    {
        let channels = rtt_init! {
            // creating rtt channels
            up: { 0: { size: 1024, mode: ChannelMode::NoBlockSkip, name: "Terminal" } }
            down: { 0: { size: 32, mode: ChannelMode::NoBlockSkip, name: "Terminal" } }
        };
                                                            // [[default.rtt.up_channels]]
        //rtt_target::set_print_channel(channels.up.0);     // format = "String"
        rtt_target::set_defmt_channel(channels.up.0);       // format = "Defmt"
                                                            // should be set accordinly in Embed.toml
    }

    // initializing rtt_target, rtos-trace and systemview backend
    #[cfg(feature = "rtos_trace")]
    {
        let channels = rtt_init! {
            // creating rtt channels - SysView should be the second
            // to be compatible with systemview_target
            up: {
                0: { size: 1024, mode: ChannelMode::NoBlockSkip, name: "Terminal" }
                1: { }
                //1: { size: 2048, name: "SysView" } 
            }
            down: {
                0: { size: 32, mode: ChannelMode::NoBlockSkip, name: "Terminal" }
                1: { }
                //1: { size: 8, name: "SysView" }
            }
        };
                                                            // [[default.rtt.up_channels]]
        //rtt_target::set_print_channel(channels.up.0);     // format = "String"
        rtt_target::set_defmt_channel(channels.up.0);       // format = "Defmt"
                                                            // should be set accordinly in Embed.toml

        SystemView::new().init();

        // although there will be the offical start in esp-rtos but
        // systemview app cannot start capturing without this call
        rtos_trace::trace::start();
    }
}