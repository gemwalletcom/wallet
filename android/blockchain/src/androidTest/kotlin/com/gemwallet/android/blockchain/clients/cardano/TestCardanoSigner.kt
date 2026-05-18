package com.gemwallet.android.blockchain.clients.cardano

import com.gemwallet.android.blockchain.includeLibs
import com.gemwallet.android.blockchain.services.SignService
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.Fee
import com.gemwallet.android.testkit.TEST_PHRASE
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.UTXO
import junit.framework.TestCase.assertEquals
import kotlinx.coroutines.runBlocking
import org.junit.Test
import wallet.core.jni.CoinType
import wallet.core.jni.HDWallet
import java.math.BigInteger

class TestCardanoSigner {
    companion object {
        init {
            includeLibs()
        }
    }

    @Test
    fun testCardanoNativeSign() {
        val privateKey = HDWallet(TEST_PHRASE, "").getKeyForCoin(CoinType.CARDANO)
        val signer = SignService()

        val sign = runBlocking {
            signer.signNativeTransfer(
                params = ConfirmParams.TransferParams.Native(
                    Chain.Cardano.asset(),
                    Account(Chain.Cardano, "addr1q9d2dxen8ywvs9yzxxn2w4mvffn797fquauvugt2ug7mfsuqj3lzdq9h0rsketzszrnfm930658swmpe7kpq53c2tmwql4rvtq", ""),
                    BigInteger.valueOf(10_000),
                    DestinationAddress("addr1q9d2dxen8ywvs9yzxxn2w4mvffn797fquauvugt2ug7mfsuqj3lzdq9h0rsketzszrnfm930658swmpe7kpq53c2tmwql4rvtq"),
                ),
                chainData = CardanoChainData(
                    utxos = listOf(
                        UTXO(
                            address = "addr1q9d2dxen8ywvs9yzxxn2w4mvffn797fquauvugt2ug7mfsuqj3lzdq9h0rsketzszrnfm930658swmpe7kpq53c2tmwql4rvtq",
                            transaction_id = "412c5a964cf4515210bf4b82f45df6521c38e1e5381f27638fc509bef6679378",
                            value = "7945975",
                            vout = 1,
                        )
                    ),
                    blockNumber = 189_992_800uL,
                ),
                finalAmount = BigInteger.valueOf(10_000),
                fee = Fee.Plain(
                    priority = FeePriority.Normal,
                    feeAssetId = AssetId(Chain.Cardano),
                    amount = BigInteger.TEN,
                    options = emptyMap(),
                ),
                privateKey.data(),
            )
        }

        assertEquals("84a40081825820412c5a964cf4515210bf4b82f45df6521c38e1e5381f27638fc509bef66" +
                "79378010182825839015aa69b33391cc8148231a6a7576c4a67e2f920e778ce216ae23db4c380947" +
                "e2680b778e16cac5010e69d962fd50f076c39f5820a470a5edc192710825839015aa69b33391cc81" +
                "48231a6a7576c4a67e2f920e778ce216ae23db4c380947e2680b778e16cac5010e69d962fd50f076" +
                "c39f5820a470a5edc1a00768570021a00029277031a0b532b80a10081825820878150b7cb71f9406" +
                "e36dcdd250e22dc943ec4525233581536497acb4f13e6705840aac7a33b86a5f8c08a3887b278646" +
                "5dbb6007eaeb9edcf9002e16ae8f5d7880c7a33cb5c8e317abf910cabbe5e8ff3f8d48b404d9fc" +
                "fd5025447c49698d01900f5f6", String(sign.first()))
    }
}
