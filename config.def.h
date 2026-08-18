#ifndef CONFIG_H
#define CONFIG_H

#include <stdint.h>
#include <stdio.h>

// The gaps surrounding the clients
static const int32_t gaps = 50;

static const char *const autostart[] = {
  "swaybg", "-i", "~/Images/Wallpapers/fiswordhd.png", NULL
};

// Use this to define your terminal command
// All command vectors follow the same logic
// static const char *termcmd[] = { "st-wl", "an-arg", "why-not-another-arg", NULL} ;
static const char *termcmd[] = { "wezterm", NULL} ;

static const char *menucmd[] = { "bemenu-run", "-p", "Run:", "-l", "10", NULL };

#endif