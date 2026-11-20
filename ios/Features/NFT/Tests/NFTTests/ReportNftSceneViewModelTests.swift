import Foundation
import GemstoneServicesTestKit
@testable import NFT
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct ReportNftSceneViewModelTests {
    @Test
    func aSubmittedReportNamesTheAssetAndTheReason() async throws {
        let service = GemCollectibleServiceMock()
        var completed = false
        let assetData = NFTAssetData.mock(collection: .mock(id: .mock()), asset: .mock(tokenId: "11871"))
        let model = ReportNftSceneViewModel(service: service, assetData: assetData, onComplete: { completed = true })

        await model.submitReport(reason: .spam)

        let report = try #require(service.reports.first)
        #expect(report.assetId == assetData.asset.id.identifier)
        #expect(report.reason == ReportReason.spam.toGem())
        #expect(completed)
    }

    @Test
    func aFailedReportLeavesTheSceneInError() async {
        let service = GemCollectibleServiceMock()
        service.reportError = AnyError("no network")
        let model = ReportNftSceneViewModel(service: service, assetData: .mock(), onComplete: nil)

        await model.submitReport(reason: .spam)

        #expect(model.state.isError)
        #expect(model.isPresentingAlertMessage?.message == AnyError("no network").localizedDescription)
    }
}
