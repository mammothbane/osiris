define tstack
	print "frame"
	x/xg $arg0
	x/xg $arg0 + 8
	if $arg0 != $arg1
		tstack *(void(**))($arg0) $arg1 
	end
end

source gdb.py
set python print-stack full
set output-radix 16
