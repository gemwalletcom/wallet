package com.gemwallet.android.ui

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.localization.string
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletSource
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletImportRequest
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.NameRecord

suspend fun GemWalletServiceInterface.importWallet(kind: GemWalletImportKind, chain: Chain?, input: String, nameRecord: NameRecord?, source: WalletSource, context: Context): GemWalletImportResult = importWallet(
    GemWalletImportRequest(
        kind = kind,
        chain = chain?.string,
        input = input,
        nameRecord = nameRecord,
        defaultName = defaultWalletName(chain?.string).string(context),
        source = source.toGem(),
    ),
)
