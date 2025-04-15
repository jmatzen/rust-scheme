;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; Test Suite for Rusty Scheme Implementation
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

;; Simple assertion helper
;; NOTE: Error handling within the test suite itself is limited.
;;       If an evaluation causes a Rust-level error, the suite might halt.
(define failed-tests 0)
(define passed-tests 0)

(define assert-equal?
  (lambda (expected actual test-name)
    (if (equal? expected actual)
      (begin
        (set! passed-tests (+ passed-tests 1))
        ;; (display "PASS: ") (display test-name) (newline) ;; Uncomment for verbose output
        #t)
      (begin
        (set! failed-tests (+ failed-tests 1))
        (display "FAIL: ") (display test-name) (newline)
        (display "  Expected: ") (display expected) (newline)
        (display "  Actual:   ") (display actual) (newline)
        #f))))

(display "--- Running Test Suite ---") (newline)

;; --- Basic Literals and Quote ---
(begin
  (display "Testing Literals and Quote...") (newline)
  (assert-equal? 123 123 "Integer literal")
  (assert-equal? #t #t "Boolean true literal")
  (assert-equal? #f #f "Boolean false literal")
  (assert-equal? "hello world" "hello world" "String literal")
  (assert-equal? '() '() "Nil literal")
  (assert-equal? 'a 'a "Symbol literal (quoted)")
  (assert-equal? '(+ 1 2) (quote (+ 1 2)) "Quoted list")
  (assert-equal? '(a b c) '(a b c) "Quoted list equality")
  (assert-equal? #t (equal? '(1 2) '(1 2)) "equal? on lists")
  (assert-equal? #f (equal? '(1 2) '(1 3)) "equal? on different lists")
)

;; --- Arithmetic ---
(begin
  (display "Testing Arithmetic...") (newline)
  (assert-equal? 6 (+ 1 2 3) "+ basic")
  (assert-equal? 0 (+ ) "+ no args")
  (assert-equal? 5 (+ 5) "+ one arg")
  (assert-equal? 5 (- 10 5) "- basic")
  (assert-equal? -10 (- 10) "- negate")
  (assert-equal? 3 (- 10 5 2) "- multiple args")
  (assert-equal? 24 (* 2 3 4) "* basic")
  (assert-equal? 1 (*) "* no args")
  (assert-equal? 5 (* 5) "* one arg")
  (assert-equal? 5 (/ 10 2) "/ basic")
  (assert-equal? 1 (/ 10 5 2) "/ multiple args")
  ;; TODO: Add tests for division by zero if error handling is implemented
)

;; --- Comparisons ---
(begin
  (display "Testing Comparisons...") (newline)
  (assert-equal? #t (= 5 5) "= equal numbers")
  (assert-equal? #f (= 5 6) "= unequal numbers")
  (assert-equal? #t (= 1 1 1 1) "= multiple equal numbers")
  (assert-equal? #f (= 1 1 2 1) "= multiple unequal numbers")
  (assert-equal? #t (< 4 5) "< less than")
  (assert-equal? #f (< 5 4) "< not less than")
  (assert-equal? #f (< 5 5) "< equal")
  (assert-equal? #t (< 1 2 3 4) "< multiple increasing")
  (assert-equal? #f (< 1 2 4 3) "< multiple not strictly increasing")
)

;; --- List Operations ---
(begin
  (display "Testing List Operations...") (newline)
  (assert-equal? '(1 2 3) (cons 1 '(2 3)) "cons basic")
  (assert-equal? '(1) (cons 1 '()) "cons onto nil")
  (assert-equal? '((1 2) 3) (cons '(1 2) '(3)) "cons list onto list")
  (assert-equal? 1 (car '(1 2 3)) "car basic")
  (assert-equal? '(2 3) (cdr '(1 2 3)) "cdr basic")
  (assert-equal? '() (cdr '(1)) "cdr single element list")
  (assert-equal? '(1 2 3) (list 1 2 3) "list basic")
  (assert-equal? '() (list) "list empty")
  (assert-equal? #t (list? '(1 2)) "list? true")
  (assert-equal? #f (list? 1) "list? false")
  (assert-equal? #t (null? '()) "null? true")
  (assert-equal? #f (null? '(1)) "null? false")
)

;; --- Special Forms (if, define, set!, begin) ---
(begin
  (display "Testing Special Forms...") (newline)
  (assert-equal? 'yes (if #t 'yes 'no) "if true branch")
  (assert-equal? 'no (if #f 'yes 'no) "if false branch")
  (assert-equal? 'yes (if (< 1 2) 'yes 'no) "if condition eval true")
  (assert-equal? 'no (if (> 1 2) 'yes 'no) "if condition eval false")
  (assert-equal? '() (if #f 'ignored) "if false branch missing (should be nil)") ; Verify implementation detail

  (define test-var 10)
  (assert-equal? 10 test-var "define basic")
  (set! test-var 20)
  (assert-equal? 20 test-var "set! basic")

  (assert-equal? 3 (begin (+ 1 1) (+ 1 2)) "begin sequence")
  (assert-equal? 5 (begin (define temp 5) temp) "begin with define")
  (assert-equal? '() (begin) "begin empty")
)

;; --- Lambda and Application ---
(begin
  (display "Testing Lambda and Application...") (newline)
  (define add1 (lambda (x) (+ x 1)))
  (assert-equal? 6 (add1 5) "lambda simple call")

  (assert-equal? 10 ((lambda (x y) (+ x y)) 3 7) "lambda immediate call")

  ;; Closure test
  (define make-adder (lambda (n) (lambda (x) (+ x n))))
  (define add5 (make-adder 5))
  (define add10 (make-adder 10))
  (assert-equal? 8 (add5 3) "closure add5")
  (assert-equal? 13 (add10 3) "closure add10")
  (assert-equal? 9 (add5 4) "closure add5 again")

  ;; Scope test
  (define x 100)
  (define lambda-scope-test
    (lambda (x) ; Parameter shadows global x
      (define y 20) ; Local define
      (+ x y)))
  (assert-equal? 15 (lambda-scope-test 5) "lambda local scope")
  (assert-equal? 100 x "lambda does not change global x")
  ;; TODO: Test if y is accessible globally (it shouldn't be) - needs error catching

)

;; --- Tail Call Optimization (TCO) ---
(begin
  (display "Testing Tail Call Optimization...") (newline)
  (define sum-to
    (lambda (n acc)
      (if (= n 0)
        acc
        (sum-to (- n 1) (+ n acc))))) ; Tail call position
  ;; Use a reasonably large number, but not excessively huge
  ;; to avoid taking too long. 10000 should be fine.
  (assert-equal? 50005000 (sum-to 10000 0) "TCO sum recursive call")

  (define count-down
    (lambda (n)
      (if (> n 0)
        (begin
          ;; (display n) (newline) ;; Uncomment to see counting
          (count-down (- n 1))) ; Tail call position
        'done)))
   (assert-equal? 'done (count-down 10000) "TCO count down")
)

;; --- Array Literals and Functions ---
(begin
  (display "Testing Arrays...") (newline)
  (define arr1 [10, "hi", #t])
  (assert-equal? #t (array? arr1) "array? true")
  (assert-equal? #f (array? '(1 2)) "array? false")
  (assert-equal? #t (equal? arr1 [10, "hi", #t]) "array literal equality")
  (assert-equal? #f (equal? arr1 [10, "hi", #f]) "array literal inequality")

  (assert-equal? 3 (array-length arr1) "array-length")
  (assert-equal? 10 (array-ref arr1 0) "array-ref first")
  (assert-equal? "hi" (array-ref arr1 1) "array-ref middle")
  (assert-equal? #t (array-ref arr1 2) "array-ref last")

  (assert-equal? '() (array-set! arr1 1 "hello") "array-set! return value")
  (assert-equal? "hello" (array-ref arr1 1) "array-ref after set!")
  (assert-equal? #t (equal? arr1 [10, "hello", #t]) "array equality after set!")

  (define arr-empty [])
  (assert-equal? 0 (array-length arr-empty) "array empty length")
  (assert-equal? #t (equal? arr-empty []) "array empty equality")

  (define arr-trailing [1, 2,])
  (assert-equal? 2 (array-length arr-trailing) "array trailing comma length")
  (assert-equal? #t (equal? arr-trailing [1, 2]) "array trailing comma equality")

  (define arr-made (make-array 3 'fill))
  (assert-equal? 3 (array-length arr-made) "make-array length")
  (assert-equal? 'fill (array-ref arr-made 0) "make-array fill value 1")
  (assert-equal? 'fill (array-ref arr-made 2) "make-array fill value 2")
)

;; --- Map Literals and Functions ---
(begin
  (display "Testing Maps...") (newline)
  (define map1 { name: "Alice", age: 30, active: #t })
  (assert-equal? #t (map? map1) "map? true")
  (assert-equal? #f (map? [1 2]) "map? false")
  (assert-equal? #t (equal? map1 { name: "Alice", age: 30, active: #t }) "map literal equality")
  (assert-equal? #t (equal? map1 { age: 30, active: #t, name: "Alice" }) "map literal equality (order)") ; Rc<RefCell<HashMap>> equality handles order
  (assert-equal? #f (equal? map1 { name: "Alice", age: 31, active: #t }) "map literal inequality")

  (assert-equal? "Alice" (map-ref map1 'name) "map-ref symbol key")
  (assert-equal? 30 (map-ref map1 'age) "map-ref number val")
  (assert-equal? #t (map-ref map1 'active) "map-ref bool val")
  (assert-equal? '() (map-ref map1 'city) "map-ref non-existent key")

  (assert-equal? '() (map-set! map1 'age 31) "map-set! return value (update)")
  (assert-equal? 31 (map-ref map1 'age) "map-ref after update set!")
  (assert-equal? '() (map-set! map1 'city "Paris") "map-set! return value (insert)")
  (assert-equal? "Paris" (map-ref map1 'city) "map-ref after insert set!")

  (define map-keys-list (map-keys map1))
  (assert-equal? #t (list? map-keys-list) "map-keys returns list")
  (assert-equal? 4 (list-length map-keys-list) "map-keys correct number") ; Need list-length helper or builtin
  ;; Checking exact keys is tricky due to order, just check type and count for now

  (define map-empty {})
  (assert-equal? 0 (list-length (map-keys map-empty)) "map empty keys") ; Needs list-length
  (assert-equal? #t (equal? map-empty {}) "map empty equality")

  (define map-trailing { a: 1, b: 2, })
  (assert-equal? 2 (list-length (map-keys map-trailing)) "map trailing comma keys") ; Needs list-length
  (assert-equal? #t (equal? map-trailing { a: 1, b: 2 }) "map trailing comma equality")

  (define map-made (make-map))
  (assert-equal? 0 (list-length (map-keys map-made)) "make-map empty") ; Needs list-length
  (map-set! map-made 'key 'value)
  (assert-equal? 'value (map-ref map-made 'key) "make-map set/ref")
)

;; --- Type Predicates ---
(begin
  (display "Testing Type Predicates...") (newline)
  (assert-equal? #t (integer? 5) "integer? true")
  (assert-equal? #f (integer? #t) "integer? false")
  (assert-equal? #t (boolean? #t) "boolean? true")
  (assert-equal? #f (boolean? 0) "boolean? false")
  (assert-equal? #t (symbol? 'abc) "symbol? true")
  (assert-equal? #f (symbol? "abc") "symbol? false")
  (assert-equal? #t (string? "abc") "string? true")
  (assert-equal? #f (string? 'abc) "string? false")
  (assert-equal? #t (procedure? +) "procedure? builtin")
  (assert-equal? #t (procedure? (lambda (x) x)) "procedure? lambda")
  (assert-equal? #f (procedure? 1) "procedure? false")
  ;; list?, null?, array?, map? tested in their respective sections
)

;; --- Need a list-length helper for map tests ---
(define list-length
  (lambda (lst)
    (if (null? lst)
        0
        (+ 1 (list-length (cdr lst))))))

;; --- Re-run Map tests requiring list-length ---
(begin
 (display "Re-Testing Maps with list-length...")(newline)
 (define map1 { name: "Alice", age: 30, active: #t })
 (map-set! map1 'city "Paris") ; Ensure it has 4 keys from previous test run
 (define map-keys-list (map-keys map1))
 (assert-equal? 4 (list-length map-keys-list) "map-keys correct number (re-test)")
 (define map-empty {})
 (assert-equal? 0 (list-length (map-keys map-empty)) "map empty keys (re-test)")
 (define map-trailing { a: 1, b: 2, })
 (assert-equal? 2 (list-length (map-keys map-trailing)) "map trailing comma keys (re-test)")
 (define map-made (make-map))
 (assert-equal? 0 (list-length (map-keys map-made)) "make-map empty (re-test)")
)


;; --- Final Summary ---
(newline)
(display "--- Test Suite Summary ---") (newline)
(display "Passed: ") (display passed-tests) (newline)
(display "Failed: ") (display failed-tests) (newline)
(if (= failed-tests 0)
    (display "All tests passed!")
    (display "Some tests failed!"))
(newline)

;; Return a value indicating success/failure maybe?
(= failed-tests 0)
