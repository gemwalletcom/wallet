package com.gemwallet.android.domains.wallet

import com.wallet.core.primitives.WalletId
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemWalletSecretKind

@Serializable
data class WalletSecretInput(val walletId: WalletId, val kind: GemWalletSecretKind)
