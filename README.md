
# Kaleidoscope-Rust

LLVM Kaleidoscope in Rust (using [Inkwell](https://github.com/TheDan64/inkwell)) through to end of Chapter 7

# Example

```c
In [#]: # Define ':' for sequencing: as a low-precedence operator that ignores operands
        # and just returns the RHS.
        def binary : 1 (x y) y;
In [#]: # Recursive fib, we could do this before.
        def fib(x)
        if (x < 3) then
            1
        else
            fib(x-1)+fib(x-2);
In [#]: # Iterative fib.
        def fibi(x)
        var a = 1, b = 1, c in
        (for i = 3, i < x in
            c = a + b :
            a = b :
            b = c) :
        b;
In [#]: fib(10);
Out[#]: 55

In [#]: fibi(10);
Out[#]: 55

In [#]: exit

; ModuleID = 'kaleidoscope'
source_filename = "kaleidoscope"

define double @"binary:"(double %x, double %y) {
entry:
  ret double %y
}

define double @fib(double %x) {
entry:
  %cmptmp = fcmp ult double %x, 3.000000e+00
  br i1 %cmptmp, label %ifcont, label %else

else:                                             ; preds = %entry
  %subtmp = fadd double %x, -1.000000e+00
  %calltmp = call double @fib(double %subtmp)
  %subtmp5 = fadd double %x, -2.000000e+00
  %calltmp6 = call double @fib(double %subtmp5)
  %addtmp = fadd double %calltmp, %calltmp6
  br label %ifcont

ifcont:                                           ; preds = %else, %entry
  %iftmp = phi double [ %addtmp, %else ], [ 1.000000e+00, %entry ]
  ret double %iftmp
}

define double @fibi(double %x) {
entry:
  br label %loop

loop:                                             ; preds = %loop, %entry
  %a.0 = phi double [ 1.000000e+00, %entry ], [ %b.0, %loop ]
  %b.0 = phi double [ 1.000000e+00, %entry ], [ %addtmp, %loop ]
  %i.0 = phi double [ 3.000000e+00, %entry ], [ %nextvar, %loop ]
  %addtmp = fadd double %a.0, %b.0
  %binop = call double @"binary:"(double %addtmp, double %b.0)
  %binop6 = call double @"binary:"(double %binop, double %addtmp)
  %cmptmp = fcmp ult double %i.0, %x
  %nextvar = fadd double %i.0, 1.000000e+00
  br i1 %cmptmp, label %loop, label %afterloop

afterloop:                                        ; preds = %loop
  %binop11 = call double @"binary:"(double 0.000000e+00, double %addtmp)
  ret double %binop11
}
```


# Notes
- For the AST type system, we decided to use an `AST` enum that holds on to all the various `struct`s. This is mentioned as good practice in the Rust programming book, largely because the number of types is limited. Intead, we could have used the `Box<dyn Trait>` duck-typing methodlogy discussed in https://doc.rust-lang.org/book/ch17-01-what-is-oo.html. But there is a performance penalty for dynamic dispatch that we don't need to pay.

# TODO
- [ ] More graceful error handling; mainly calling `unwrap` everywhere now
- [ ] Add some built-in externs to handle `putchar` and `printf` with double
