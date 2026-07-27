package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockSwapParams
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemTransferAmountException
import java.math.BigInteger

class CalculateTransferAmountImplTest {

    private val asset = mockAssetSolana()
    private val token = mockAssetSolanaUSDC()
    private val account = mockAccount(asset.id.chain)
    private val fee = BigInteger.valueOf(5_000L)

    @Test
    fun `input carries max flag and swap minimum`() {
        val amount = BigInteger.valueOf(10_000_000L)
        val feeAssetInfo = mockAssetInfo(
            asset = asset,
            balance = AssetBalance.create(asset = asset, available = "100000000"),
        )

        val transfer = transferAmountInput(
            params = ConfirmParams.Builder(asset, account, amount, useMaxAmount = true)
                .transfer(DestinationAddress("recipient")),
            availableValue = amount,
            feeAssetInfo = feeAssetInfo,
            fee = fee,
        )
        assertTrue(transfer.isMaxAmount)
        assertNull(transfer.minimumValue)
        assertEquals(amount.toString(), transfer.value)
        assertEquals("100000000", transfer.feeAssetBalance)

        val swap = transferAmountInput(
            params = mockSwapParams(
                from = account,
                fromAsset = asset,
                fromAmount = amount,
                minFromAmount = BigInteger.valueOf(1_000L),
                toAsset = token,
            ),
            availableValue = amount,
            feeAssetInfo = feeAssetInfo,
            fee = fee,
        )
        assertEquals("1000", swap.minimumValue)
    }

    @Test
    fun `error mapping picks the reported asset`() {
        val balance = GemTransferAmountException.InsufficientBalance(
            assetId = asset.id.toIdentifier(),
            required = "10005000",
            available = "10000000",
        ).toConfirmError(asset, asset) as ConfirmError.InsufficientBalance
        assertEquals(asset, balance.asset)
        assertEquals(BigInteger.valueOf(10_005_000L), balance.requirement.required)
        assertEquals(BigInteger.valueOf(10_000_000L), balance.requirement.available)

        val tokenBalance = GemTransferAmountException.InsufficientBalance(
            assetId = token.id.toIdentifier(),
            required = "1",
            available = "0",
        ).toConfirmError(token, asset) as ConfirmError.InsufficientBalance
        assertEquals(token, tokenBalance.asset)

        val networkFee = GemTransferAmountException.InsufficientNetworkFee(
            assetId = asset.id.toIdentifier(),
            required = "5000",
            available = "1000",
        ).toConfirmError(token, asset) as ConfirmError.InsufficientFee
        assertEquals(asset.id.chain, networkFee.chain)
        assertEquals(BigInteger.valueOf(1_000L), networkFee.requirement.available)

        val minimum = GemTransferAmountException.MinimumAccountBalanceTooLow(
            assetId = asset.id.toIdentifier(),
            required = "890880",
            available = "495000",
        ).toConfirmError(asset, asset) as ConfirmError.MinimumAccountBalanceTooLow
        assertEquals(asset, minimum.asset)
        assertEquals(BigInteger.valueOf(890_880L), minimum.requirement.required)
    }
}
