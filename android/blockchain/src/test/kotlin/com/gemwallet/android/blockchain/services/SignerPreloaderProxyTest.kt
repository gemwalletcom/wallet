package com.gemwallet.android.blockchain.services

import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetTempoUSDCe
import com.gemwallet.android.testkit.mockGemTransactionLoadFee
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.ScanTransaction
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemGatewayInterface
import uniffi.gemstone.GemTransactionData
import uniffi.gemstone.GemTransactionLoadInput
import uniffi.gemstone.GemTransactionLoadMetadata
import java.math.BigInteger

class SignerPreloaderProxyTest {

    private val gateway = mockk<GemGatewayInterface>()
    private val subject = SignerPreloaderProxy(gateway)

    @Test
    fun preload_loadsOnlySelectedPriorityAndKeepsAllFeeRates() = runBlocking {
        val params = transferParams()
        val metadata = evmMetadata()
        val feeRates = listOf(
            GemFeeRate(FeePriority.Normal.string, GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "3")),
            GemFeeRate(FeePriority.Fast.string, GemGasPriceType.Eip1559(gasPrice = "4", priorityFee = "5")),
        )
        val loadInput = slot<GemTransactionLoadInput>()

        coEvery { gateway.getTransactionPreload(any(), any()) } returns metadata
        coEvery { gateway.getFeeRates(any(), any()) } returns feeRates
        coEvery { gateway.getTransactionLoad(any(), capture(loadInput)) } returns transactionData(feeRates[1].gasPriceType)

        val result = subject.preload(params, FeeSelection.Preset(FeePriority.Normal))

        assertEquals(feeRates, result.feeRates)
        assertEquals(FeePriority.Normal, result.fee().priority)
        assertEquals(BigInteger("21000"), result.fee().amount)
        assertEquals(feeRates[0].gasPriceType, loadInput.captured.gasPrice)
        coVerify(exactly = 1) { gateway.getTransactionPreload(any(), any()) }
        coVerify(exactly = 1) { gateway.getFeeRates(any(), any()) }
        coVerify(exactly = 1) { gateway.getTransactionLoad(any(), any()) }
    }

    @Test
    fun preload_fallsBackToFirstAvailableValidPriority() = runBlocking {
        val params = transferParams()
        val metadata = evmMetadata()
        val feeRates = listOf(
            GemFeeRate(priority = "unsupported", gasPriceType = GemGasPriceType.Eip1559(gasPrice = "1", priorityFee = "1")),
            GemFeeRate(FeePriority.Fast.string, GemGasPriceType.Eip1559(gasPrice = "4", priorityFee = "5")),
        )
        val loadInput = slot<GemTransactionLoadInput>()

        coEvery { gateway.getTransactionPreload(any(), any()) } returns metadata
        coEvery { gateway.getFeeRates(any(), any()) } returns feeRates
        coEvery { gateway.getTransactionLoad(any(), capture(loadInput)) } returns transactionData(feeRates[1].gasPriceType)

        val result = subject.preload(params, FeeSelection.Preset(FeePriority.Normal))

        assertEquals(listOf(feeRates[1]), result.feeRates)
        assertEquals(FeePriority.Fast, result.fee().priority)
        assertEquals(feeRates[1].gasPriceType, loadInput.captured.gasPrice)
        coVerify(exactly = 1) { gateway.getTransactionLoad(any(), any()) }
    }

    @Test
    fun preload_usesFeeAssetIdFromGatewayWhenPresent() = runBlocking {
        val feeAsset = mockAssetTempoUSDCe()
        val params = transferParams(feeAsset)
        stubPreload()
        coEvery { gateway.getTransactionLoad(any(), any()) } returns GemTransactionData(
            fee = mockGemTransactionLoadFee(feeAssetId = feeAsset.id),
            metadata = evmMetadata(),
        )

        val result = subject.preload(params, FeeSelection.Preset(FeePriority.Normal))

        assertEquals(feeAsset.id, result.fee().feeAssetId)
    }

    @Test
    fun preload_blocksMaliciousScan() = runBlocking {
        val params = transferParams()
        val subject = SignerPreloaderProxy(
            gateway = gateway,
            scanTransaction = { ScanTransaction(isMalicious = true, isMemoRequired = false) },
        )

        stubPreload()
        stubTransactionLoad()

        val error = runCatching { subject.preload(params, FeeSelection.Preset(FeePriority.Normal)) }.exceptionOrNull()

        assertEquals(ConfirmError.ScanTransactionMalicious, error)
    }

    @Test
    fun preload_blocksMemoRequiredScanWithoutMemo() = runBlocking {
        val params = transferParams()
        val subject = SignerPreloaderProxy(
            gateway = gateway,
            scanTransaction = { ScanTransaction(isMalicious = false, isMemoRequired = true) },
        )

        stubPreload()
        stubTransactionLoad()

        val error = runCatching { subject.preload(params, FeeSelection.Preset(FeePriority.Normal)) }.exceptionOrNull()

        assertEquals(params.asset.symbol, (error as? ConfirmError.ScanTransactionMemoRequired)?.symbol)
    }

    private fun transferParams(asset: Asset = mockAssetEthereum()): ConfirmParams {
        return ConfirmParams.Builder(
            asset = asset,
            from = sender(asset),
            amount = BigInteger("1000000000000000"),
        ).transfer(
            destination = DestinationAddress("0xrecipient"),
        )
    }

    private fun sender(
        asset: Asset,
        derivationPath: String = "m/44'/60'/0'/0/0",
    ) = mockAccount(
        chain = asset.id.chain,
        address = "0xsender",
        derivationPath = derivationPath,
    )

    private fun stubTransactionLoad() {
        coEvery { gateway.getTransactionLoad(any(), any()) } returns
            transactionData(GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "3"))
    }

    private fun stubPreload() {
        coEvery { gateway.getTransactionPreload(any(), any()) } returns evmMetadata()
        coEvery { gateway.getFeeRates(any(), any()) } returns listOf(
            GemFeeRate(FeePriority.Normal.string, GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "3")),
        )
    }

    private fun evmMetadata() = GemTransactionLoadMetadata.Evm(
        nonce = 7u,
        chainId = 1u,
        contractCall = null,
    )

    private fun transactionData(gasPriceType: GemGasPriceType) = GemTransactionData(
        fee = mockGemTransactionLoadFee(
            fee = "21000",
            gasPriceType = gasPriceType,
        ),
        metadata = evmMetadata(),
    )
}
