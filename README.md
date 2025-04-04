## The BIKE

The CCM-450 Adventure was a small series of enduro-style motorcycles built in
Bolton, England, by the Clews family business. The engine was bought from BMW,
possibly from the unused stock of the former BMW 450X. The frame (without
welding), tank and fairings are the company's own design and manufacture, other
parts (forks, wheels, etc) are from good quality manufacturers, many others are
Husquarna parts.

## The problem

As I was probably the furthest distance from Bolton to buy such a motorcycle, I
didn't have the chance to rely on the factory service. Besides, I have never
taken any of my vehicles, cars or motorcycles, to a mechanic or to a dealership.
This is probably the reason why I have used each of them for many hundreds of
thousands of kilometres, literally for the rest of their lives.

I always get the necessary diagnostic tools for all my vehicles, because
without them, their maintenance and repair is simple not possible today. For
newer vehicle types, this often means "smart acquiring" the factory SW, but if
there is anything else available on the market at a reasonable price, I buy it.
With the CCM450 there was no chance of that. I also looked at the Husquarna 449
and 511, which I have information use the same ECU (Keihin KMSK16). I couldn't
find factory or non-factory solution available. The non-factory diagnostic
solutions that were already available for BMW motorcycles using a similar ECU
did not work for these Husquarna models.

## My solution

Eventually I was forced to crack, reverse-engineer my own ECU. In the end, I
could only get the built-in secret access code by brute-force method. Back when
I did this I shared the details on a cafehusky forum. The first page of these
series can be found here:

http://www.cafehusky.com/threads/449-511-communication-with-ecu.39593/page-2#post-537495

This community was also suffering from not being able to get any diagnostic
tools for their motorcycles. Initially, I only suspected that what I learned on
my CCM450 motorcycle would be relevant here. Later it proved to be true. These
motorcycles have exactly the same ECU, SW and keys as my motorcycle.

## An implementation plan

Now, many years later, I have had the time to put the experience into SW form.
And it is also a good opportunity to immerse myself in a new chip family and
today's modern embedded technologies. The goal is a tiny device that can be
easily integrated into this motorcycle, supporting all the interrogation and
programming functions of the ECU on the one hand, and wi-fi communication with
the outside world on the other.

The planned milestones: develop a minimum code base for an esp32c3 chip, which
can be a starting point for further own developments. This would contain only
the initial runtime (boot), interrupt, panic and exception handling, the basic
console and logging functions.

In this and the following stages, we explicitly aim to minimize the size of the
code. Therefore, logging is only done using defmt crate, otherwise only simple
println is possible. But even the latter can be turned off completely. In
addition to defmt, the whole panic management of the "core' crate and especially
the related formatting functions can also be turned off. Of course, all these
size optimization solutions should only be turned on in a well-tested
application.

In further steps, we will build a minimal embassy environment and then a Wi-Fi
stack on top of it. Due to size optimization, I plan that only the ip protocol
and a simple udp server will run above the physical wifi layer. Finally, using
the UART peripheral of the esp32c3, which is connected to the ECU by a L9637D
signal level converter chip, I implement a KWP2000 protocol-based, ECU-specific
communication layer for the UDP serve as a gateway of all ECU communication.

## Considerations for implementation

I would like to follow the painful, long but ultimately most
knowledge-providing path of starting almost all code, my own, as well as used
external crates, from scratch. Although this may seem like copy-paste
technology at first, if you understand each "paste" and build even the external
codes step by step, you will eventually be able to see the details and the big
picture as well.
