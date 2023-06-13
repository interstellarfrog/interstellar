handle SIGTRAP stop
break *0x00100000
break *0x00130000
break *0x800000a580
break panic
break kernel_main
break init
break src/main.rs:1:0
tui layout split