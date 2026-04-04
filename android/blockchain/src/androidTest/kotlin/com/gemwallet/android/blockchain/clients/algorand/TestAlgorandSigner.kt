package com.gemwallet.android.blockchain.clients.algorand

import com.gemwallet.android.blockchain.includeLibs
import com.gemwallet.android.blockchain.services.SignService
import com.gemwallet.android.blockchain.testPhrase
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.Fee
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import junit.framework.TestCase.assertEquals
import kotlinx.coroutines.runBlocking
import org.junit.Test
import wallet.core.jni.CoinType
import wallet.core.jni.HDWallet
import java.math.BigInteger

class TestAlgorandSigner {

    companion object {
        init {
            includeLibs()
        }
    }

    @Test
    fun testAlgorandNativeSign() {
        val hdWallet = HDWallet(testPhrase, "")
        val privateKey = hdWallet.getKeyForCoin(CoinType.ALGORAND)
        val from = hdWallet.getAddressForCoin(CoinType.ALGORAND)
        val signer = SignService()

        val sign = runBlocking {
            signer.signNativeTransfer(
                params = ConfirmParams.TransferParams.Native(
                    Chain.Algorand.asset(),
                    Account(Chain.Algorand, from, ""),
                    BigInteger.valueOf(10_000_000),
                    DestinationAddress(from),
                ),
                chainData = AlgorandChainData(
                    sequence = 46932581UL,
                    block = "wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8=",
                    chainId = "mainnet-v1.0",
                ),
                finalAmount = BigInteger.valueOf(10_000_000),
                fee = Fee.Plain(
                    priority = FeePriority.Normal,
                    feeAssetId = AssetId(Chain.Algorand),
                    amount = BigInteger.TEN,
                    options = emptyMap(),
                ),
                privateKey.data(),
            )
        }

        assertEquals("82a3736967c44049c1faef9ca28e835bb980e84e2d6f6eaf866828abf8ce51f1820d04ea" +
                "87623069603104aaadcdea598dd6e77f8dbe40c7f9638485cfeda1609b810e23ce1604a374786e8" +
                "9a3616d74ce00989680a36665650aa26676ce02cc2265a367656eac6d61696e6e65742d76312e30" +
                "a26768c420c061c4d8fc1dbdded2d7604be4568e3f6d041987ac37bde4b620b5ab39248adfa26c7" +
                "6ce02cc264da3726376c42033b2e013d23fae630e2182f241c513530bd72ea9680558c034218ab0" +
                "863d47b8a3736e64c42033b2e013d23fae630e2182f241c513530bd72ea9680558c034218ab0863" +
                "d47b8a474797065a3706179", String(sign.first()))
    }
}
