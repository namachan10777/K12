(let f 
  (lambda (x)
    (if (less x 2)
      1
      (add (f (sub x 1) (sub x 2)))))
  (f 5))
