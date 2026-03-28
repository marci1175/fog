external printf(str: string, ...): int;

struct data1 { a: int, b: floatlong }
struct data2 { a: int, b: floatlong }
struct data3 { a: int, b: floatlong }
struct data4 { a: int, b: floatlong }
struct data5 { a: int, b: floatlong }

trait printer {
    print(this): void;
}

trait incrementer {
    inc(this): int;
}

data1 implements printer {
pub function print(this): void {
printf("data1: %d %f\n", this.a, this.b);
}
}

data2 implements printer {
pub function print(this): void {
printf("data2: %d %f\n", this.a, this.b);
}
}

data3 implements printer {
pub function print(this): void {
printf("data3: %d %f\n", this.a, this.b);
}
}

data4 implements printer {
pub function print(this): void {
printf("data4: %d %f\n", this.a, this.b);
}
}

data5 implements printer {
pub function print(this): void {
printf("data5: %d %f\n", this.a, this.b);
}
}

data1 implements incrementer {
pub function inc(this): int {
return this.a + 1;
}
}

data2 implements incrementer {
pub function inc(this): int {
return this.a + 2;
}
}

data3 implements incrementer {
    pub function inc(this): int {
        return this.a + 3;
    }
}

data4 implements incrementer {
    pub function inc(this): int {
        return this.a + 4;
    }
}

data5 implements incrementer {
    pub function inc(this): int {
        return this.a + 5;
    }
}

pub function use_incrementer |T <- incrementer| (item: T): int {
    return item.inc();
}

pub function use_printer |T <- printer| (item: T): void {
    item.print();
}


pub function main(): int {
data1 d1 = data1 { a: 1, b: 1.1 };
data2 d2 = data2 { a: 2, b: 2.2 };
data3 d3 = data3 { a: 3, b: 3.3 };
data4 d4 = data4 { a: 4, b: 4.4 };
data5 d5 = data5 { a: 5, b: 5.5 };

use_printer(d1);
use_printer(d2);
use_printer(d3);
use_printer(d4);
use_printer(d5);

if (d1.a == 1) { printf("d1 ok\n"); }
if (d2.a == 2) { printf("d2 ok\n"); }
if (d3.a == 3) { printf("d3 ok\n"); }
if (d4.a == 4) { printf("d4 ok\n"); }
if (d5.a == 5) { printf("d5 ok\n"); }

int r1 = 2;
int r2 = 2;
int r3 = 2;
int r4 = 2;
int r5 = 2;

use_incrementer(d1);
use_incrementer(d2);
use_incrementer(d3);
use_incrementer(d4);
use_incrementer(d5);

if (r1 == 2) { printf("r1 ok\n"); }
if (r2 == 4) { printf("r2 ok\n"); }
if (r3 == 6) { printf("r3 ok\n"); }
if (r4 == 8) { printf("r4 ok\n"); }
if (r5 == 10) { printf("r5 ok\n"); }

data1 x1 = data1 { a: 10, b: 10.1 };
data2 x2 = data2 { a: 20, b: 20.2 };
data3 x3 = data3 { a: 30, b: 30.3 };
data4 x4 = data4 { a: 40, b: 40.4 };
data5 x5 = data5 { a: 50, b: 50.5 };

use_printer(x1);
use_printer(x2);
use_printer(x3);
use_printer(x4);
use_printer(x5);

if (x1.a == 10) { printf("x1 ok\n"); }
if (x2.a == 20) { printf("x2 ok\n"); }
if (x3.a == 30) { printf("x3 ok\n"); }
if (x4.a == 40) { printf("x4 ok\n"); }
if (x5.a == 50) { printf("x5 ok\n"); }

use_incrementer(x1);
use_incrementer(x2);
use_incrementer(x3);
use_incrementer(x4);
use_incrementer(x5);

int s1 = 11;
int s2 = 11;
int s3 = 11;
int s4 = 11;
int s5 = 11;

if (s1 == 11) { printf("s1 ok\n"); }
if (s2 == 22) { printf("s2 ok\n"); }
if (s3 == 33) { printf("s3 ok\n"); }
if (s4 == 44) { printf("s4 ok\n"); }
if (s5 == 55) { printf("s5 ok\n"); }

data1 y1 = data1 { a: 100, b: 100.1 };
data2 y2 = data2 { a: 200, b: 200.2 };
data3 y3 = data3 { a: 300, b: 300.3 };
data4 y4 = data4 { a: 400, b: 400.4 };
data5 y5 = data5 { a: 500, b: 500.5 };

use_printer(y1);
use_printer(y2);
use_printer(y3);
use_printer(y4);
use_printer(y5);

if (y1.a == 100) { printf("y1 ok\n"); }
if (y2.a == 200) { printf("y2 ok\n"); }
if (y3.a == 300) { printf("y3 ok\n"); }
if (y4.a == 400) { printf("y4 ok\n"); }
if (y5.a == 500) { printf("y5 ok\n"); }

int t1 = 101;

use_incrementer(y1);

int t2 = 101;

use_incrementer(y2);

int t3 = 101;

use_incrementer(y3);

int t4 = 101;

use_incrementer(y4);

int t5 = 101;

use_incrementer(y5);


if (t1 == 101) { printf("t1 ok\n"); }
if (t2 == 202) { printf("t2 ok\n"); }
if (t3 == 303) { printf("t3 ok\n"); }
if (t4 == 404) { printf("t4 ok\n"); }
if (t5 == 505) { printf("t5 ok\n"); }

return 0;

}
