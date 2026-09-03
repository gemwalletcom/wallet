package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import uniffi.gemstone.GemRecipient
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetBalance
import uniffi.gemstone.GemTransactionInputType
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.math.BigInteger

class AmountTransferProviderTest {

    private val asset = mockAssetCosmos()
    private val assetInfo = mockAssetInfo(asset = asset)
    private val getAssetInfo = mockk<GetAssetInfo> {
        every { this@mockk.invoke(asset.id) } returns flowOf(assetInfo)
    }
    private val scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob())
    private val params = AmountParams.Transfer(
        assetId = asset.id,
        destination = GemRecipient(address = "to", name = null),
        memo = "memo",
    )

    private fun makeProvider() = AmountTransferProvider(
        params = params,
        getAssetInfo = getAssetInfo,
        scope = scope,
    )

    @Test
    fun `title is Send`() {
        assertEquals(AmountTitle.Send, makeProvider().title)
    }

    @Test
    fun `canChangeValue and canSwitchInputType are both true`() {
        val provider = makeProvider()
        assertTrue(provider.canChangeValue.value)
        assertTrue(provider.canSwitchInputType)
    }

    @Test
    fun `minimumValue and reserveForFee are zero`() {
        val provider = makeProvider()
        assertEquals(BigInteger.ZERO, provider.minimumValue.value)
        assertEquals(BigInteger.ZERO, provider.reserveForFee.value)
    }

    @Test
    fun `buildTransfer produces a transfer with destination and memo`() = runBlocking {
        val provider = makeProvider()
        provider.assetInfo.filterNotNull().first()
        val transfer = provider.buildTransfer(amount = Crypto(BigInteger.ONE), isMax = false)
        assertTrue(transfer.inputType is GemTransactionInputType.Transfer)
        assertEquals(BigInteger.ONE, transfer.value)
        assertEquals("to", transfer.recipient.address)
        assertEquals("memo", transfer.recipient.memo)
    }

    @Test
    fun `deposit has Deposit title`() {
        val provider = AmountTransferProvider(
            params = AmountParams.Deposit(asset.id),
            getAssetInfo = getAssetInfo,
            scope = scope,
        )
        assertEquals(AmountTitle.Deposit, provider.title)
    }

    @Test
    fun `withdraw has Withdraw title`() {
        val provider = AmountTransferProvider(
            params = AmountParams.Withdraw(asset.id),
            getAssetInfo = getAssetInfo,
            scope = scope,
        )
        assertEquals(AmountTitle.Withdraw, provider.title)
    }

    @Test
    fun `withdraw availableBalance uses withdrawable, not available`() = runBlocking {
        val info = mockAssetInfo(
            asset = asset,
            balance = AssetBalance.create(asset = asset, available = "9000000", withdrawable = "5000000"),
        )
        val getInfo = mockk<GetAssetInfo> {
            every { this@mockk.invoke(asset.id) } returns flowOf(info)
        }
        val provider = AmountTransferProvider(
            params = AmountParams.Withdraw(asset.id),
            getAssetInfo = getInfo,
            scope = scope,
        )
        assertEquals(BigInteger("5000000"), provider.availableBalance.first { it != BigInteger.ZERO })
    }

    @Test
    fun `withdraw builds a withdrawal to the own address`() = runBlocking {
        val provider = AmountTransferProvider(
            params = AmountParams.Withdraw(asset.id),
            getAssetInfo = getAssetInfo,
            scope = scope,
        )
        val owner = provider.assetInfo.filterNotNull().first().owner
        val transfer = provider.buildTransfer(amount = Crypto(BigInteger.ONE), isMax = false)
        assertTrue(transfer.inputType is GemTransactionInputType.Withdrawal)
        assertEquals(owner?.address, transfer.recipient.address)
    }
}
