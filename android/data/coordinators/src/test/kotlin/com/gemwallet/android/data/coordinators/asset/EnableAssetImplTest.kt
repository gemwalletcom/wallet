package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockWalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemBalanceService

class EnableAssetImplTest {

    private val balanceService = mockk<GemBalanceService> {
        coEvery { setAssetsEnabled(any(), any(), any()) } returns Unit
    }
    private val subject = EnableAssetImpl(balanceService)

    @Test
    fun enablesAssetsThroughCore() = runTest {
        val asset = mockAsset()
        val walletId = mockWalletId()

        subject(walletId, asset.id)

        coVerify { balanceService.setAssetsEnabled(walletId.id, listOf(asset.id.toIdentifier()), true) }
    }
}
