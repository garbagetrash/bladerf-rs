/* Save to a file, e.g. boilerplate.c, and then compile:
 * $ gcc boilerplate.c -o libbladeRF_example_boilerplate -lbladeRF
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "libbladeRF.h"


int sync_rx_meta_now_example(struct bladerf *dev,
                             int16_t *samples,
                             unsigned int samples_len,
                             unsigned int rx_count,
                             unsigned int timeout_ms);

/* The RX and TX channels are configured independently for these parameters */
struct channel_config {
    bladerf_channel channel;
    unsigned int frequency;
    unsigned int bandwidth;
    unsigned int samplerate;
    int gain;
};

int configure_channel(struct bladerf *dev, struct channel_config *c)
{
    int status;

    status = bladerf_set_frequency(dev, c->channel, c->frequency);
    if (status != 0) {
        fprintf(stderr, "Failed to set frequency = %u: %s\n", c->frequency,
                bladerf_strerror(status));
        return status;
    }

    status = bladerf_set_sample_rate(dev, c->channel, c->samplerate, NULL);
    if (status != 0) {
        fprintf(stderr, "Failed to set samplerate = %u: %s\n", c->samplerate,
                bladerf_strerror(status));
        return status;
    }

    status = bladerf_set_bandwidth(dev, c->channel, c->bandwidth, NULL);
    if (status != 0) {
        fprintf(stderr, "Failed to set bandwidth = %u: %s\n", c->bandwidth,
                bladerf_strerror(status));
        return status;
    }

    status = bladerf_set_gain(dev, c->channel, c->gain);
    if (status != 0) {
        fprintf(stderr, "Failed to set gain: %s\n", bladerf_strerror(status));
        return status;
    }

    return status;
}

/* Usage:
 *   libbladeRF_example_boilerplate [serial #]
 *
 * If a serial number is supplied, the program will attempt to open the
 * device with the provided serial number.
 *
 * Otherwise, the first available device will be used.
 */
int main(int argc, char *argv[])
{
    int status;
    struct channel_config config;

    struct bladerf *dev = NULL;
    struct bladerf_devinfo dev_info;

    /* Initialize the information used to identify the desired device
     * to all wildcard (i.e., "any device") values */
    bladerf_init_devinfo(&dev_info);

    /* Request a device with the provided serial number.
     * Invalid strings should simply fail to match a device. */
    if (argc >= 2) {
        strncpy(dev_info.serial, argv[1], sizeof(dev_info.serial) - 1);
    }

    status = bladerf_open_with_devinfo(&dev, &dev_info);
    if (status != 0) {
        fprintf(stderr, "Unable to open device: %s\n",
                bladerf_strerror(status));

        return 1;
    }

    /* Set up RX channel parameters */
    config.channel    = BLADERF_CHANNEL_RX(0);
    config.frequency  = 910000000;
    config.bandwidth  = 50000000;
    config.samplerate = 61440000;
    config.gain       = 39;

    status = configure_channel(dev, &config);
    if (status != 0) {
        fprintf(stderr, "Failed to configure RX channel. Exiting.\n");
        goto out;
    }

    /* Application code goes here.
     *
     * Don't forget to call bladerf_enable_module() before attempting to
     * transmit or receive samples!
     */
    /* Configure the device's RX for use with the sync interface.
     * SC16 Q11 samples *with* metadata are used. */
    const unsigned int num_buffers = 64;
    const unsigned int buffer_size = 16384;
    const unsigned int num_transfers = 16;
    const unsigned int timeout_ms = 0;

    status = bladerf_sync_config(dev, BLADERF_RX_X1,
                                 BLADERF_FORMAT_SC16_Q11_META, num_buffers,
                                 buffer_size, num_transfers, timeout_ms);
    if (status != 0) {
        fprintf(stderr, "Failed to configure RX sync interface: %s\n",
                bladerf_strerror(status));
        goto out;
    }

    // Enable the RF frontend after sync configuration
    status = bladerf_enable_module(dev, BLADERF_RX, true);
    if (status != 0) {
        fprintf(stderr, "Failed to enable RX: %s\n", bladerf_strerror(status));
        goto out;
    }

    const unsigned int nsamples = 8192;
    int16_t* buffer = (int16_t*)malloc(2*sizeof(int16_t)*nsamples);
    if (buffer == NULL) { perror("malloc"); return BLADERF_ERR_MEM; }
    unsigned int rx_count = 1000000000;
    sync_rx_meta_now_example(
        dev,
        buffer,
        nsamples,
        rx_count,
        timeout_ms
    );

out:
    bladerf_close(dev);
    free(buffer);
    return status;
}

int sync_rx_meta_now_example(struct bladerf *dev,
                             int16_t *samples,
                             unsigned int samples_len,
                             unsigned int rx_count,
                             unsigned int timeout_ms)
{
    int status = 0;
    struct bladerf_metadata meta;
    unsigned int i;
    /* Perform a read immediately, and have the bladerf_sync_rx function
     * provide the timestamp of the read samples */
    memset(&meta, 0, sizeof(meta));
    meta.flags = BLADERF_META_FLAG_RX_NOW;

    printf("waiting 2 seconds for things to settle... ");
    fflush(stdout);
    usleep(2000000);
    printf("go!\n");
    fflush(stdout);
    /* Receive samples and do work on them */
    int nfailed = 0;
    for (i = 0; i < rx_count && status == 0; i++) {
        if ((i+1) % 1000 == 0) { fprintf(stdout, "i: %d, nfailed: %d\n", i, nfailed); fflush(stdout); }
        status = bladerf_sync_rx(dev, samples, samples_len, &meta, timeout_ms);
        if (status != 0) {
            fprintf(stderr, "RX \"now\" failed: %s\n\n",
                    bladerf_strerror(status));
        } else if (meta.status & BLADERF_META_STATUS_OVERRUN) {
            fprintf(stderr, "Overrun detected. %u valid samples were read\n",
                    meta.actual_count);
            fprintf(stderr, "at t=0x%016" PRIx64 "\n", meta.timestamp);
            nfailed++;
            if (nfailed > 5) {
                break;
            }
        } else {
            // Spin while things are working...
        }
    }
    return status;
}
