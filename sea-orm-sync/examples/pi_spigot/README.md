# Pi Spigot Algorithm

This program implements the Rabinowitz-Wagon decimal spigot algorithm to generate
1 million digits of pi without big integer libraries.

It shows how to easily use SeaORM to a program to make it resumable and persistent.
Run the program, press Ctrl-C at any time to pause, and run it again.
Instead of restarting from beginning, it will continue from exactly where it left off.
It also saves the pi digits to a SQLite table.

This is definitely not the fastest pi computation algorithm, but the simplicity and
streaming nature makes it the best for demonstrating a long running process.