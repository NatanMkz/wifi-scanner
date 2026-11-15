use super::Scanner;
use crate::domain::BssBruto;
use std::{ffi::c_void, ptr, slice, thread, time::Duration};
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::NetworkManagement::WiFi::*;

pub struct WlanScanner;

impl Scanner for WlanScanner {
    fn escanear(&self) -> Result<Vec<BssBruto>, String> {
        let mut resultado = Vec::new();
        unsafe {
            let mut versao = 0u32;
            let mut cliente = HANDLE::default();
            if WlanOpenHandle(2, None, &mut versao, &mut cliente) != 0 {
                return Err("não consegui abrir o serviço WLAN".into());
            }

            let mut lista: *mut WLAN_INTERFACE_INFO_LIST = ptr::null_mut();
            if WlanEnumInterfaces(cliente, None, &mut lista) != 0 || lista.is_null() {
                WlanCloseHandle(cliente, None);
                return Err("não consegui listar as interfaces Wi-Fi".into());
            }

            let n = (*lista).dwNumberOfItems as usize;
            let interfaces = slice::from_raw_parts((*lista).InterfaceInfo.as_ptr(), n);

            // Pede um novo scan e espera o driver terminar
            for itf in interfaces {
                WlanScan(cliente, &itf.InterfaceGuid, None, None, None);
            }
            thread::sleep(Duration::from_secs(4));

            for itf in interfaces {
                let mut bss: *mut WLAN_BSS_LIST = ptr::null_mut();
                let r = WlanGetNetworkBssList(
                    cliente,
                    &itf.InterfaceGuid,
                    None,
                    dot11_BSS_type_any,
                    BOOL(0),
                    None,
                    &mut bss,
                );
                if r != 0 || bss.is_null() {
                    continue;
                }

                let qtd = (*bss).dwNumberOfItems as usize;
                let entradas = slice::from_raw_parts((*bss).wlanBssEntries.as_ptr(), qtd);
                for e in entradas {
                    // Os IEs ficam no mesmo bloco de memória, a ulIeOffset bytes do início da entrada
                    let ie_ptr =
                        (e as *const WLAN_BSS_ENTRY as *const u8).add(e.ulIeOffset as usize);
                    let ies = slice::from_raw_parts(ie_ptr, e.ulIeSize as usize).to_vec();
                    let len = (e.dot11Ssid.uSSIDLength as usize).min(32);

                    // Copiamos tudo para tipos Rust antes de liberar a memória do Windows
                    resultado.push(BssBruto {
                        ssid: e.dot11Ssid.ucSSID[..len].to_vec(),
                        bssid: e.dot11Bssid,
                        rssi: e.lRssi,
                        freq_khz: e.ulChCenterFrequency,
                        capability: e.usCapabilityInformation,
                        ies,
                    });
                }
                WlanFreeMemory(bss as *const c_void);
            }

            WlanFreeMemory(lista as *const c_void);
            WlanCloseHandle(cliente, None);
        }
        Ok(resultado)
    }
}
