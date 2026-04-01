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
    public var nftDataList: [NFTData] { query.value }

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public var wallet: Wallet

    public init(wallet: Wallet) {
        self.wallet = wallet
        query = ObservableQuery(NFTRequest(walletId: wallet.walletId, filter: .unverified), initialValue: [])
    }

    public var title: String { Localized.Asset.Verification.unverified }

    public var content: CollectionsContent {
        CollectionsContent(items: nftDataList.map { buildGridItem(from: $0) })
    }
}
