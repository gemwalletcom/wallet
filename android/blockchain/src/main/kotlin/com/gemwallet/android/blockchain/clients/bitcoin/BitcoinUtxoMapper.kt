package com.gemwallet.android.blockchain.clients.bitcoin

import com.gemwallet.android.math.decodeHex
import com.google.protobuf.ByteString
import com.wallet.core.primitives.UTXO
import wallet.core.jni.BitcoinScript
import wallet.core.jni.CoinType
import wallet.core.jni.proto.Bitcoin

fun List<UTXO>.getUtxoTransactions(address: String, coinType: CoinType): List<Bitcoin.UnspentTransaction> {
    return map { utxo ->
        Bitcoin.UnspentTransaction.newBuilder().apply {
            val hash = utxo.transaction_id.decodeHex()
            hash.reverse()
            this.outPoint = Bitcoin.OutPoint.newBuilder().apply {
                this.hash = ByteString.copyFrom(hash)
                this.index = utxo.vout
            }.build()
            this.amount = utxo.value.toLong()
            this.script = ByteString.copyFrom(
                BitcoinScript.lockScriptForAddress(address, coinType).data()
            )
        }.build()
    }
}
