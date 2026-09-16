MEMORY
{
/* RAM allocatet fully to the application core, unlike the embassy example since this project doesn't use the RISC-V coprocessor. So no RAM needs reserving for it. */
FLASH : ORIGIN = 0x00000000, LENGTH = 1524K
RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

