import Foundation
import GemstoneServicesTestKit
@testable import NFT
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct ReportNftViewModelTests {
    @Test
    func aSubmittedReportNamesTheCollectionTheAssetAndTheReason() async throws {
        let service = GemCollectibleServiceMock()
        var completed = false
        let assetData = NFTAssetData.mock(collection: .mock(id: .mock()), asset: .mock(tokenId: "11871"))
        let model = ReportNftViewModel(service: service, assetData: assetData, onComplete: { completed = true })

        await model.submitReport(reason: "spam")

        let report = try #require(service.reports.first)
        #expect(report.collectionId == assetData.collection.id.identifier)
        #expect(report.assetId == assetData.asset.id.identifier)
        #expect(report.reason == "spam")
        #expect(completed)
    }

    @Test
    func aFailedReportLeavesTheSceneInError() async {
        let service = GemCollectibleServiceMock()
        service.reportError = AnyError("no network")
        let model = ReportNftViewModel(service: service, assetData: .mock(), onComplete: nil)

        await model.submitReport(reason: "spam")

        if case .data = model.state {
            Issue.record("a failed report must not read as submitted")
        }
    }
}
