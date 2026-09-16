package com.gemwallet.android.features.bridge.localization

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Chain
import androidx.annotation.StringRes
import uniffi.gemstone.GemConnectionDetailRow
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.MessageType

@Composable
internal fun Throwable.walletConnectMessage(): String? = when (this) {
    is GemWalletConnectException.UnsupportedChains -> stringResource(R.string.errors_connections_unsupported_chain)
    is GemWalletConnectException.InvalidOrigin -> stringResource(R.string.errors_connections_malicious_origin)
    is GemWalletConnectException.UnsupportedWallets -> stringResource(R.string.errors_connections_no_supported_wallets)
    is GemWalletConnectException.Service -> msg
    else -> null
}

@Composable
internal fun MessageType.string(): String = when (this) {
    MessageType.SIWE -> stringResource(R.string.common_sign_in_with, Chain.Ethereum.networkName())
    MessageType.SIWS -> stringResource(R.string.common_sign_in_with, Chain.Solana.networkName())
    MessageType.TEXT, MessageType.EIP712 -> stringResource(R.string.transfer_review_request)
}

@StringRes
internal fun GemConnectionDetailRow.stringRes(): Int = when (this) {
    GemConnectionDetailRow.WALLET -> R.string.common_wallet
    GemConnectionDetailRow.DATE -> R.string.transaction_date
}
