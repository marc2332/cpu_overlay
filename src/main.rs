#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{sync::{Arc}, thread::{self, sleep}, time::Duration, rc::Rc, borrow::Borrow, cell::RefCell};
use futures::executor::block_on;
use tokio::{sync::{mpsc::{channel, Sender}, Mutex}};

use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};
use rui::*;

#[derive(Deserialize, Debug)]
struct SensorResult {
    Name: String,
    Value: f32,
}

#[tokio::main]
async fn main(){
    
    let (tx, rx) = channel::<SensorResult>(1);

    let rx = Arc::new(Mutex::new(rx));
    
    tokio::task::spawn_blocking( move || {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::with_namespace_path("root/OpenHardwareMonitor",com_con.into()).unwrap();

        loop {
            let results: Vec<SensorResult> = wmi_con.raw_query("SELECT Name, Value FROM Sensor where Name = 'CPU Package' and SensorType = 'Temperature'").unwrap();
    
            for res in results {
                block_on(tx.send(res)).unwrap();
            }

            sleep(Duration::from_millis(500));

        }
    });

    let mounted = RefCell::new(false);

    rui(state(0, move |temp| {

        let te = temp.clone();
        let rx = rx.clone();

       if !*mounted.borrow() {
        tokio::spawn(async move {
            let mut rx = rx.lock().await;

            loop {
                let res = rx.recv().await;

                if let Some(res) = res{
                    te.with_mut(|t| *t = res.Value as i32);
                }
            }
        });
        *mounted.borrow_mut() = true;
       }
   
        vstack((
            vstack((
                text(&format!("{:?}", temp.get()))
                    .font_size(30)
                    .padding(Auto),
                text("CPU Temp")
                    .font_size(20)
            ))
            .size(LocalSize::new(50.0, 100.0)),
        ))
    }));

}