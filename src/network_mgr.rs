//! RPC to network management (naive with http)
use run_script::ScriptOptions;
use rustcracker::model::network_interface::NetworkInterface;

use crate::error::{VmManageError, VmManageResult};

/* 
    Manage network configuration.
    End point: NetworkInterface.
    Need: guest_mac, host_dev_name, iface_id, rx_rate_limiter, tx_rate_limiter.

    Make a network interface on the host machine.
    Configure network interface(run the script)
    Manage IP address for ethernet card and MAC address for all microVMs.
    (Using customed DHCP)

    Boot stage: guest_mac, host_dev_name configured.
    Could be modified when running: iface_id, rx_rate_limiter, tx_rate_limiter.

Workflow:
1. Manually
    Create tun/tap device on host.
    Configure tun/tap device. (Give it a tun/tap IP address)
    Configure routing on the host.
    After-boot:
    Configure network in the guest.

2. Using OVN.
    Automatically manage network configuration (subnet, ip and routing).
    Configure network in the guest.

Provides:
    RPC(Remote Procedure Call) to the network manager. (Binary
    listening on a specific port of the host.)
    Network manager: interact with OVN.
    (No crate with full functionality managing OVN for now.)
*/


fn mock(tap_id: u32) -> VmManageResult<()> {
    let (_code, _output, _error) = run_script::run_script!(
        r#"
        TAP_DEV="tap$1"
        HOST_IFACE="eth$1"
        TAP_IP="172.16.0.1"
        MASK_SHORT="/30"

        # Setup network interface
        sudo ip link del "$TAP_DEV" 2> /dev/null || true
        sudo ip tuntap add dev "$TAP_DEV" mode tap
        sudo ip addr add "${TAP_IP}${MASK_SHORT}" dev "$TAP_DEV"
        sudo ip link set dev "$TAP_DEV" up

        # Enable ip forwarding
        sudo sh -c "echo 1 > /proc/sys/net/ipv4/ip_forward"

        # Set up microVM internet access
        sudo iptables -t nat -D POSTROUTING -o "$HOST_IFACE" -j MASQUERADE || true
        sudo iptables -D FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT \
            || true
        sudo iptables -D FORWARD -i "$TAP_DEV" -o "$HOST_IFACE" -j ACCEPT || true
        sudo iptables -t nat -A POSTROUTING -o "$HOST_IFACE" -j MASQUERADE
        sudo iptables -I FORWARD 1 -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
        sudo iptables -I FORWARD 1 -i "$TAP_DEV" -o "$HOST_IFACE" -j ACCEPT
        "#,
        &vec![format!("{}", tap_id)],
        &ScriptOptions::new()
    ).map_err(|_| VmManageError::NetworkError)?;
    Ok(())
}


pub async fn create_network_interface(tap_id: u32) -> VmManageResult<NetworkInterface> {
    // let url = format!("{}{}", pool.network_mgr_addr, "/api/v1/netif");
    mock(tap_id)?;
    let host_dev_name = format!("tap{tap_id}");
    
    let net_if = NetworkInterface {
        // Fixed in rootfs.
        guest_mac: Some(format!("AA:FC:00:00:00:0{}", tap_id)),
        iface_id: tap_id.to_string(),
        host_dev_name: host_dev_name.into(),
        rx_rate_limiter: None,
        tx_rate_limiter: None,
    };

    Ok(net_if)
}