// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemCollectibleServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class ReportNftSceneViewModel {
    private let service: any GemCollectibleServiceProtocol
    private let assetData: NFTAssetData
    private let onComplete: VoidAction

    var state: StateViewType<Bool> = .noData
    var isPresentingAlertMessage: AlertMessage?

    let reasons = ReportReason.allCases

    init(service: any GemCollectibleServiceProtocol, assetData: NFTAssetData, onComplete: VoidAction) {
        self.service = service
        self.assetData = assetData
        self.onComplete = onComplete
    }

    func listItem(for reason: ReportReason) -> ListItemModel {
        ListItemModel(title: reason.title)
    }

    var title: String {
        Localized.Nft.Report.reportButtonTitle
    }

    var progressMessage: String {
        Localized.Common.loading
    }

    func submitReport(reason: ReportReason) async {
        state = .loading
        do {
            try await service.report(assetId: assetData.asset.id.identifier, reason: reason.toGem())
            state = .data(true)
            onComplete?()
        } catch {
            state = .error(error)
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }
}
