// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class UnverifiedCollectionsViewModel: CollectionsViewable, Sendable {
    public let query: ObservableQuery<NFTRequest>

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public init(wallet: Wallet) {
        query = ObservableQuery(NFTRequest(walletId: wallet.id, filter: .unverified), initialValue: [])
    }

    public var title: String {
        Localized.Asset.Verification.unverified
    }

    public var content: CollectionsContent {
        CollectionsContent(items: query.value.map { buildGridItem(from: $0) })
    }
}
