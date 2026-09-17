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

        model.submitReport(reason: "spam")
        try await settle { if case .data = model.state { return true } else { return false } }

        let report = try #require(service.reports.first)
        #expect(report.collectionId == assetData.collection.id.identifier)
        #expect(report.assetId == assetData.asset.id.identifier)
        #expect(report.reason == "spam")
        #expect(completed)
    }

    @Test
    func aFailedReportLeavesTheSceneInError() async throws {
        let service = GemCollectibleServiceMock()
        service.reportError = AnyError("no network")
        let model = ReportNftViewModel(service: service, assetData: .mock(), onComplete: nil)

        model.submitReport(reason: "spam")
        try await settle { if case .error = model.state { return true } else { return false } }

        if case .data = model.state {
            Issue.record("a failed report must not read as submitted")
        }
    }

    private func settle(until condition: @MainActor () -> Bool) async throws {
        for _ in 0 ..< 100 {
            if condition() { return }
            try await Task.sleep(for: .milliseconds(10))
        }
        Issue.record("the report never settled")
    }
}
