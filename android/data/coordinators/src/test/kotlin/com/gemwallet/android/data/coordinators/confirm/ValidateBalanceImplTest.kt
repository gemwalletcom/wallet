package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.ext.getMinimumAccountBalance
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.ChainSignData
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockDelegation
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.FeePriority
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import org.junit.After
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import java.math.BigInteger

class ValidateBalanceImplTest {

    private val asset = mockAssetSolana()
    private val account = mockAccount(asset.id.chain)
    private val feeAmount = BigInteger.valueOf(5_000L)

    @Before
    fun setUp() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { Chain.Solana.getMinimumAccountBalance() } returns 890_880L
    }

    @After
    fun tearDown() {
        unmockkStatic("com.gemwallet.android.ext.ChainKt")
    }

    @Test
    fun `stake withdraw passes`() {
        val stakeAmount = BigInteger.valueOf(649_953_059L)
        val params = ConfirmParams.Builder(asset, account, stakeAmount)
            .withdraw(mockDelegation(asset.id, DelegationState.AwaitingWithdrawal, stakeAmount.toString()))
        val info = assetInfo(walletAvailable = "5000000")

        validate(params, finalAmount = stakeAmount, info = info, assetBalance = stakeAmount)
    }

    @Test
    fun `transfer below minimum throws`() {
        val transferAmount = BigInteger.valueOf(10_000_000L)
        val params = ConfirmParams.Builder(asset, account, transferAmount)
            .transfer(DestinationAddress("recipient"))
        val info = assetInfo(walletAvailable = "10500000")

        assertThrows(ConfirmError.MinimumAccountBalanceTooLow::class.java) {
            validate(params, finalAmount = transferAmount, info = info, assetBalance = BigInteger("10500000"))
        }
    }

    @Test
    fun `transfer with insufficient balance throws`() {
        val transferAmount = BigInteger.valueOf(10_000_000L)
        val params = ConfirmParams.Builder(asset, account, transferAmount)
            .transfer(DestinationAddress("recipient"))
        val info = assetInfo(walletAvailable = "10000000")

        assertThrows(ConfirmError.InsufficientBalance::class.java) {
            validate(params, finalAmount = transferAmount, info = info, assetBalance = BigInteger("10000000"))
        }
    }

    @Test
    fun `withdraw with insufficient fee throws`() {
        val stakeAmount = BigInteger.valueOf(600_000_000L)
        val params = ConfirmParams.Builder(asset, account, stakeAmount)
            .withdraw(mockDelegation(asset.id, DelegationState.AwaitingWithdrawal, stakeAmount.toString()))
        val info = assetInfo(walletAvailable = "1000")

        assertThrows(ConfirmError.InsufficientFee::class.java) {
            validate(params, finalAmount = stakeAmount, info = info, assetBalance = stakeAmount)
        }
    }

    @Test
    fun `token transfer skips minimum check when fee asset would drop below minimum`() {
        val token = mockAssetSolanaUSDC()
        val tokenAmount = BigInteger.valueOf(10_000_000L)
        val tokenAccount = mockAccount(token.id.chain)
        val params = ConfirmParams.Builder(token, tokenAccount, tokenAmount)
            .transfer(DestinationAddress("recipient"))
        val tokenInfo = mockAssetInfo(
            asset = token,
            balance = AssetBalance.create(asset = token, available = tokenAmount.toString()),
        )
        val solInfo = assetInfo(walletAvailable = "10000")

        ValidateBalanceImpl().invoke(
            signerParams = SignerParams(
                input = params,
                selectedData = SignerParams.Data(
                    fee = Fee.Plain(asset.id, FeePriority.Normal, feeAmount, emptyMap()),
                    chainData = mockk<ChainSignData>(),
                ),
                feeRates = emptyList(),
                finalAmount = tokenAmount,
            ),
            assetInfo = tokenInfo,
            feeAssetInfo = solInfo,
            assetBalance = tokenAmount,
        )
    }

    @Test
    fun `transfer with max amount skips minimum check`() {
        val transferAmount = BigInteger.valueOf(9_990_000L)
        val params = ConfirmParams.Builder(asset, account, transferAmount, useMaxAmount = true)
            .transfer(DestinationAddress("recipient"))
        val info = assetInfo(walletAvailable = "10000000")

        validate(params, finalAmount = transferAmount, info = info, assetBalance = BigInteger("10000000"))
    }

    private fun assetInfo(walletAvailable: String) = mockAssetInfo(
        asset = asset,
        balance = AssetBalance.create(asset = asset, available = walletAvailable),
    )

    private fun validate(
        params: ConfirmParams,
        finalAmount: BigInteger,
        info: com.gemwallet.android.model.AssetInfo,
        assetBalance: BigInteger,
    ) {
        ValidateBalanceImpl().invoke(
            signerParams = SignerParams(
                input = params,
                selectedData = SignerParams.Data(
                    fee = Fee.Plain(asset.id, FeePriority.Normal, feeAmount, emptyMap()),
                    chainData = mockk<ChainSignData>(),
                ),
                feeRates = emptyList(),
                finalAmount = finalAmount,
            ),
            assetInfo = info,
            feeAssetInfo = info,
            assetBalance = assetBalance,
        )
    }
}
