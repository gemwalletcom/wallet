// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletImportKind
import struct Gemstone.GemWalletImportRequest
import enum Gemstone.GemWalletImportResult
import protocol Gemstone.GemWalletServiceProtocol
import struct Gemstone.NameRecord
import GemstonePrimitives
import Primitives
import PrimitivesComponents

extension GemWalletServiceProtocol {
    func importWallet(
        kind: GemWalletImportKind,
        chain: Chain?,
        input: String,
        nameRecord: NameRecord?,
        source: Primitives.WalletSource,
    ) async throws -> GemWalletImportResult {
        try await importWallet(
            request: GemWalletImportRequest(
                kind: kind,
                chain: chain?.toGem(),
                input: input,
                nameRecord: nameRecord,
                defaultName: defaultWalletName(chain: chain?.toGem()).text,
                source: source.toGem(),
            ),
        )
    }
}
