mod descriptor;
mod context;
mod xhci_usb_device;
// 命令管理器、事件管理器和插槽管理器等模块。
pub(super) mod extended_capabilities;
pub(super) mod registers;
pub(super) mod xhci_command_manager;
pub(super) mod xhci_event_manager;
pub(super) mod xhci_roothub;
pub(super) mod xhci_slot_manager;

// 定义了命令环、事件环和暂存区等内部使用的模块。
pub(crate) mod command_ring;
pub(crate) mod event_ring;
pub(super) mod scratchpad;
mod usb;
mod usb_audio;
mod usb_device;

#[derive(Debug, PartialEq)]
enum USBSpeed {
    USBSpeedLow,
    USBSpeedFull,
    USBSpeedHigh,
    USBSpeedSuper,
    USBSpeedUnknown,
}

impl From<u8> for USBSpeed {
    fn from(value: u8) -> Self {
        match Some(value) {
            Some(1) => USBSpeed::USBSpeedFull,
            Some(2) => USBSpeed::USBSpeedLow,
            Some(3) => USBSpeed::USBSpeedHigh,
            Some(4) => USBSpeed::USBSpeedSuper,
            Some(_) => USBSpeed::USBSpeedUnknown,
            None => USBSpeed::USBSpeedUnknown,
        }
    }
}

// XHCI配置事件环大小为256个TRB(Transfer Request Block)。
const XHCI_CONFIG_EVENT_RING_SIZE: usize = 256;

// 控制TRB的位掩码定义。
const XHCI_TRB_CONTROL_C: usize = 1 << 0; // 表示TRB的完成控制位。
const XHCI_LINK_TRB_CONTROL_TC: usize = 1 << 1; // 表示TRB的链接控制位。

// TRB类型控制位的偏移。
const XHCI_TRB_CONTROL_TRB_TYPE_SHIFT: usize = 10;

// 事件TRB的状态完成代码的位移。
const XHCI_EVENT_TRB_STATUS_COMPLETION_CODE_SHIFT: usize = 24;

// 传输事件TRB的传输长度掩码。
const XHCI_TRANSFER_EVENT_TRB_STATUS_TRB_TRANSFER_LENGTH_MASK: usize = 0xFFFFFF;

// 命令完成事件TRB的控制位，插槽ID的位移。
const XHCI_CMD_COMPLETION_EVENT_TRB_CONTROL_SLOTID_SHIFT: usize = 24;

// 传输事件TRB的控制位，端点ID的掩码。
const XHCI_TRANSFER_EVENT_TRB_CONTROL_ENDPOINTID_MASK: usize = 0x1F << 16;

// 传输事件TRB的控制位，端点ID的位移。
const XHCI_TRANSFER_EVENT_TRB_CONTROL_ENDPOINTID_SHIFT: usize = 16;

// 端口状态事件TRB的参数1，端口ID的位移。
const XHCI_PORT_STATUS_EVENT_TRB_PARAMETER1_PORTID_SHIFT: usize = 24;

// XHCI的IMODI配置值，用于轮询间隔。
const XHCI_CONFIG_IMODI: u16 = 500;

// XHCI的配置中最大端口数量。
const XHCI_CONFIG_MAX_PORTS: usize = 5;

// XHCI的配置中最大插槽数量。
const XHCI_CONFIG_MAX_SLOTS: usize = 64;

// TODO: 确定DMA地址。
//const DMA_ADDRESS: usize = 0xfd50_0000;

// TODO: 修正虚拟地址。
//TODO FIX VIRTUAL ADDRESS
