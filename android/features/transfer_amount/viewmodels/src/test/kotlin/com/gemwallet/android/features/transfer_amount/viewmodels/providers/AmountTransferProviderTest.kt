package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountTransfer
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import uniffi.gemstone.GemAmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetBalance
import uniffi.gemstone.TransactionInputType
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.testkit.mockAmountParamsTransfer
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockGemTransferData
import io.mockk.coEvery
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
    private val params = mockAmountParamsTransfer(assetId = asset.id, memo = "memo")

    private val transfers = mutableListOf<GemAmountTransfer>()
    private val service = mockk<GemAmountServiceInterface> {
        coEvery { transferData(any(), capture(transfers), any(), any()) } answers {
            mockGemTransferData(inputType = TransactionInputType.Transfer(firstArg()), value = thirdArg(), useMaxAmount = arg(3))
        }
    }

    private fun makeProvider(params: AmountParams = this.params) = AmountTransferProvider(
        params = params,
        service = service,
        getAssetInfo = getAssetInfo,
        scope = scope,
    )

    @Test
    fun `title is Send`() {
        assertEquals(GemAmountTitle.Send, makeProvider().title.value)
    }

    @Test
    fun `only a send switches the input type`() {
        assertTrue(makeProvider().amountType.value?.canSwitchInputType() == true)
        assertEquals(false, makeProvider(AmountParams.Deposit(asset.id)).amountType.value?.canSwitchInputType())
    }

    @Test
    fun `Core decides the prefilled amount`() {
        assertEquals("1.5", makeProvider(params.copy(amount = "1.5")).prefilledAmount)
        assertEquals(null, makeProvider().prefilledAmount)
        assertEquals(null, makeProvider(AmountParams.Deposit(asset.id)).prefilledAmount)
        assertEquals(null, makeProvider(AmountParams.Withdraw(asset.id)).prefilledAmount)
    }

    @Test
    fun `buildTransfer hands Core a send with the destination and memo`() = runBlocking {
        val provider = makeProvider()
        provider.assetInfo.filterNotNull().first()
        val transfer = provider.buildTransfer(amount = Crypto(BigInteger.ONE), isMax = false)
        assertEquals(BigInteger.ONE, transfer.value)
        val send = transfers.single() as GemAmountTransfer.Send
        assertEquals("to", send.payment.recipient.address)
        assertEquals("memo", send.payment.recipient.memo)
    }

    @Test
    fun `deposit has Deposit title`() {
        assertEquals(GemAmountTitle.Deposit, makeProvider(AmountParams.Deposit(asset.id)).title.value)
    }

    @Test
    fun `withdraw has Withdraw title`() {
        assertEquals(GemAmountTitle.Withdraw, makeProvider(AmountParams.Withdraw(asset.id)).title.value)
    }

    @Test
    fun `withdraw availableBalance uses withdrawable, not available`() = runBlocking {
        val info = mockAssetInfo(
            asset = asset,
            balance = AssetBalance.create(asset = asset, available = BigInteger("9000000"), withdrawable = BigInteger("5000000")),
        )
        val getInfo = mockk<GetAssetInfo> {
            every { this@mockk.invoke(asset.id) } returns flowOf(info)
        }
        val provider = AmountTransferProvider(
            params = AmountParams.Withdraw(asset.id),
            service = service,
            getAssetInfo = getInfo,
            scope = scope,
        )
        assertEquals(BigInteger("5000000"), provider.input.filterNotNull().first().availableValue)
    }

    @Test
    fun `deposit and withdraw hand Core their kind and the max flag`() = runBlocking {
        val deposit = makeProvider(AmountParams.Deposit(asset.id))
        deposit.assetInfo.filterNotNull().first()
        deposit.buildTransfer(amount = Crypto(BigInteger.TEN), isMax = true)
        val withdraw = makeProvider(AmountParams.Withdraw(asset.id))
        withdraw.assetInfo.filterNotNull().first()
        val transfer = withdraw.buildTransfer(amount = Crypto(BigInteger.ONE), isMax = false)

        assertEquals(listOf(GemAmountTransfer.Deposit, GemAmountTransfer.Withdraw), transfers)
        assertEquals(BigInteger.ONE, transfer.value)
    }
}
