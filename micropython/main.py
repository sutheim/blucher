import nrf24l01test


from machine import Pin
import utime

led = Pin(25, Pin.OUT)
led.low()

for i in range(10):
   led.toggle()
   utime.sleep(0.1)
   

nrf24l01test.master()