Nerve agent using an Android device via adb to do anything.

**Requirements**

* Developer mode must be enabled on the device (model specific, but usually the process is to go into settings, about phone, tap the build number 7 times, then go back and enable developer mode).
* ADB installed on the host computer ( `brew install android-platform-tools` ).
* Device must be connected to the host computer via USB or ADB must be [configured to connect wirelessly](https://www.androidpolice.com/use-wireless-adb-android-phone/).
* For the screenshot tool the model must support visual inputs.

Run with:

```bash
nerve run examples/android-agent --task 'take a selfie and send it to jessica'
```