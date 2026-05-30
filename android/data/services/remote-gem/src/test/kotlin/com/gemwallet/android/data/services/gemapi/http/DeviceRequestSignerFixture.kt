package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.data.services.gemapi.DeviceKeyPairFixture

internal fun mockDeviceRequestSigner(
    currentTimeMillis: () -> Long = { 123L },
): DeviceRequestSigner = DeviceRequestSigner(
    getDeviceId = mockGetDeviceId(),
    currentTimeMillis = currentTimeMillis,
)

internal fun mockGetDeviceId(): GetDeviceId = MockGetDeviceId(
    deviceId = DeviceKeyPairFixture.publicKeyHex,
    deviceKey = DeviceKeyPairFixture.privateKeyHex,
)

private class MockGetDeviceId(
    private val deviceId: String,
    private val deviceKey: String,
) : GetDeviceId {
    override fun getDeviceId(): String = deviceId

    override fun getDeviceKey(): String = deviceKey
}
