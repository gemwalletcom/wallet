// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import protocol Gemstone.GemCollectibleServiceProtocol
import Components
import Foundation
import Localization
import GemstoneServices
import Primitives

@Observable
@MainActor
public final class ReportNftViewModel {
    private let service: any GemCollectibleServiceProtocol
    private let assetData: NFTAssetData
    private let onComplete: VoidAction

    var state: StateViewType<Bool> = .noData

    let reasons = ReportReasonViewModel.allCases

    init(service: any GemCollectibleServiceProtocol, assetData: NFTAssetData, onComplete: VoidAction) {
        self.service = service
        self.assetData = assetData
        self.onComplete = onComplete
    }

    var title: String {
        Localized.Nft.Report.reportButtonTitle
    }

    var progressMessage: String {
        Localized.Common.loading
    }

    func submitReport(reason: String) {
        state = .loading
        Task {
            do {
                try await service.report(report: ReportNft(
                    collectionId: assetData.collection.id.identifier,
                    assetId: assetData.asset.id.identifier,
                    reason: reason,
                ).json())
                state = .data(true)
                onComplete?()
            } catch {
                debugLog("Report NFT error: \(error)")
                state = .error(error)
            }
        }
    }
}
