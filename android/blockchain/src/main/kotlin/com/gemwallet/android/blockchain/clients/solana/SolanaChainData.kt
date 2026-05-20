package com.gemwallet.android.blockchain.clients.solana

import com.gemwallet.android.model.ChainSignData
import com.wallet.core.primitives.SolanaNftStandard
import com.wallet.core.primitives.SolanaNftStandardCoreInner
import com.wallet.core.primitives.SolanaNftStandardProgrammableNonFungibleInner
import com.wallet.core.primitives.SolanaTokenProgramId
import uniffi.gemstone.GemTransactionLoadMetadata

data class SolanaChainData(
    val blockHash: String,
    val senderTokenAddress: String?,
    val recipientTokenAddress: String?,
    val tokenProgram: SolanaTokenProgramId?,
    val nft: SolanaNftStandard? = null,
) : ChainSignData {
    override fun toDto(): GemTransactionLoadMetadata {
        return GemTransactionLoadMetadata.Solana(
            blockHash = blockHash,
            senderTokenAddress = senderTokenAddress,
            recipientTokenAddress = recipientTokenAddress,
            tokenProgram = tokenProgram?.toUniffi(),
            nft = nft?.toUniffi(),
        )
    }
}

fun GemTransactionLoadMetadata.Solana.toChainData(): SolanaChainData {
    return SolanaChainData(
        blockHash = blockHash,
        senderTokenAddress = senderTokenAddress,
        recipientTokenAddress = recipientTokenAddress,
        tokenProgram = tokenProgram?.toPrimitives(),
        nft = nft?.toPrimitives(),
    )
}

private fun SolanaTokenProgramId.toUniffi(): uniffi.gemstone.SolanaTokenProgramId = when (this) {
    SolanaTokenProgramId.Token -> uniffi.gemstone.SolanaTokenProgramId.TOKEN
    SolanaTokenProgramId.Token2022 -> uniffi.gemstone.SolanaTokenProgramId.TOKEN2022
}

private fun uniffi.gemstone.SolanaTokenProgramId.toPrimitives(): SolanaTokenProgramId = when (this) {
    uniffi.gemstone.SolanaTokenProgramId.TOKEN -> SolanaTokenProgramId.Token
    uniffi.gemstone.SolanaTokenProgramId.TOKEN2022 -> SolanaTokenProgramId.Token2022
}

private fun SolanaNftStandard.toUniffi(): uniffi.gemstone.SolanaNftStandard = when (this) {
    is SolanaNftStandard.NonFungible -> uniffi.gemstone.SolanaNftStandard.NonFungible
    is SolanaNftStandard.ProgrammableNonFungible -> uniffi.gemstone.SolanaNftStandard.ProgrammableNonFungible(data.rule_set)
    is SolanaNftStandard.Core -> uniffi.gemstone.SolanaNftStandard.Core(data.collection)
}

private fun uniffi.gemstone.SolanaNftStandard.toPrimitives(): SolanaNftStandard = when (this) {
    is uniffi.gemstone.SolanaNftStandard.NonFungible -> SolanaNftStandard.NonFungible
    is uniffi.gemstone.SolanaNftStandard.ProgrammableNonFungible -> SolanaNftStandard.ProgrammableNonFungible(
        SolanaNftStandardProgrammableNonFungibleInner(rule_set = ruleSet)
    )
    is uniffi.gemstone.SolanaNftStandard.Core -> SolanaNftStandard.Core(
        SolanaNftStandardCoreInner(collection = collection)
    )
}
