#ifndef EMPIERO_HDA_SPEAKER_AMP_H
#define EMPIERO_HDA_SPEAKER_AMP_H

/*
 * Enable the analog speaker amp on the lab Dell ALC3204 (SSID 1028:09c1).
 * Mixer writes alone cannot do this: mainline has no 0x09c1 quirk, so GPIO
 * stays off. Skip any other codec. Never fail the caller.
 */
void empiieros_hda_enable_speaker_amp(void);

#endif
