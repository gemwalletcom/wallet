package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.data.repositories.transactions.TransactionBalanceService
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
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
    private val balanceService = mockk<TransactionBalanceService> {
        coEvery { getBalance(any(), any<AmountParams>(), any(), any()) } returns BigInteger("1000000")
    }
    private val scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob())
    private val params = AmountParams.Transfer(
        assetId = asset.id,
        destination = DestinationAddress(address = "to", name = null),
        memo = "memo",
    )

    private fun makeProvider() = AmountTransferProvider(
        params = params,
        getAssetInfo = getAssetInfo,
        transactionBalanceService = balanceService,
        scope = scope,
    )

    @Test
    fun `title is Send`() {
        assertEquals(AmountTitle.Send, makeProvider().title)
    }

    @Test
    fun `canChangeValue and canSwitchInputType are both true`() {
        val provider = makeProvider()
        assertTrue(provider.canChangeValue)
        assertTrue(provider.canSwitchInputType)
    }

    @Test
    fun `minimumValue and reserveForFee are zero`() {
        val provider = makeProvider()
        assertEquals(BigInteger.ZERO, provider.minimumValue.value)
        assertEquals(BigInteger.ZERO, provider.reserveForFee)
    }

    @Test
    fun `buildConfirmParams produces TransferParams with destination and memo`() = runBlocking {
        val provider = makeProvider()
        provider.assetInfo.filterNotNull().first()
        val confirm = provider.buildConfirmParams(amount = Crypto(BigInteger.ONE), isMax = false)
        assertTrue(confirm is ConfirmParams.TransferParams)
        confirm as ConfirmParams.TransferParams
        assertEquals(BigInteger.ONE, confirm.amount)
        assertEquals("to", confirm.destination.address)
        assertEquals("memo", confirm.memo)
    }

    @Test
    fun `deposit has Deposit title`() {
        val provider = AmountTransferProvider(
            params = AmountParams.Deposit(asset.id),
            getAssetInfo = getAssetInfo,
            transactionBalanceService = balanceService,
            scope = scope,
        )
        assertEquals(AmountTitle.Deposit, provider.title)
    }

    @Test
    fun `withdraw has Withdraw title`() {
        val provider = AmountTransferProvider(
            params = AmountParams.Withdraw(asset.id),
            getAssetInfo = getAssetInfo,
            transactionBalanceService = balanceService,
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
            transactionBalanceService = balanceService,
            scope = scope,
        )
        assertEquals(BigInteger("5000000"), provider.availableBalance.first { it != BigInteger.ZERO })
    }

    @Test
    fun `withdraw buildConfirmParams produces Withdrawal to own address`() = runBlocking {
        val provider = AmountTransferProvider(
            params = AmountParams.Withdraw(asset.id),
            getAssetInfo = getAssetInfo,
            transactionBalanceService = balanceService,
            scope = scope,
        )
        val owner = provider.assetInfo.filterNotNull().first().owner
        val confirm = provider.buildConfirmParams(amount = Crypto(BigInteger.ONE), isMax = false)
        assertTrue(confirm is ConfirmParams.TransferParams.Withdrawal)
        confirm as ConfirmParams.TransferParams.Withdrawal
        assertEquals(owner?.address, confirm.destination.address)
    }
}
