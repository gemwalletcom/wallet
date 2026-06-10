package com.gemwallet.android.features.bridge.viewmodels.model

import uniffi.gemstone.WalletConnectionVerificationStatus
import org.junit.Assert.assertEquals
import org.junit.Test

class WalletConnectOriginVerifierTest {

    @Test
    fun scamRejectsInvalidAndMalicious() {
        assertEquals(false, OriginVerification(WalletConnectionVerificationStatus.UNKNOWN).isScam)
        assertEquals(false, OriginVerification(WalletConnectionVerificationStatus.VERIFIED).isScam)
        assertEquals(true, OriginVerification(WalletConnectionVerificationStatus.INVALID).isScam)
        assertEquals(true, OriginVerification(WalletConnectionVerificationStatus.MALICIOUS).isScam)
    }
}
