package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.swap.coordinators.GetSwapQuoteData
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.ext.nowSeconds
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.math.fromHex
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.Config
import uniffi.gemstone.FetchQuoteData
import uniffi.gemstone.GemSwapQuoteData
import uniffi.gemstone.GemSwapper
import uniffi.gemstone.MessageSigner
import uniffi.gemstone.Permit2Data
import uniffi.gemstone.Permit2Detail
import uniffi.gemstone.PermitSingle
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage
import uniffi.gemstone.SwapperQuote
import uniffi.gemstone.permit2DataToEip712Json

class GetSwapQuoteDataImpl(
    private val gemSwapper: GemSwapper,
    private val passwordStore: PasswordStore,
    private val signMessageOperator: GemSignMessageOperator,
) : GetSwapQuoteData {

    override suspend fun invoke(quote: SwapperQuote, wallet: Wallet): GemSwapQuoteData {
        val permit = gemSwapper.getPermit2ForQuote(quote = quote)

        if (permit == null) {
            return gemSwapper.getQuoteData(quote, FetchQuoteData.None)
        }

        val permit2Single = permit2Single(
            token = permit.token,
            spender = permit.spender,
            value = permit.value,
            nonce = permit.permit2Nonce,
        )
        val chain = checkNotNull(quote.request.fromAsset.id.toAssetId()?.chain) {
            "Swap quote has invalid asset id"
        }
        val permit2Json = permit2DataToEip712Json(
            chain = chain.string,
            data = permit2Single,
            contract = permit.permit2Contract,
        )
        val signer = MessageSigner(
            SignMessage(chain = chain.string, signType = SignDigestType.EIP712, data = permit2Json.toByteArray())
        )
        val signature = try {
            signMessageOperator.sign(signer, wallet, passwordStore.getPassword(key = wallet.id.id))
        } finally {
            signer.close()
        }.fromHex()
        val permitData = Permit2Data(permit2Single, signature)
        return gemSwapper.getQuoteData(quote, FetchQuoteData.Permit2(permitData))
    }

    private fun permit2Single(token: String, spender: String, value: String, nonce: ULong): PermitSingle {
        val config = Config().getSwapConfig()
        val now = nowSeconds()
        return PermitSingle(
            details = Permit2Detail(
                token = token,
                amount = value,
                expiration = now + config.permit2Expiration,
                nonce = nonce,
            ),
            spender = spender,
            sigDeadline = now + config.permit2SigDeadline,
        )
    }
}
