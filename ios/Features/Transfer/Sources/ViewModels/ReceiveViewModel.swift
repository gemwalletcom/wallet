import Components
import Foundation
import class Gemstone.GemAddressService
import protocol Gemstone.GemReceiveServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ReceiveViewModel: Sendable {
    var qrSize: CGFloat {
        UIDevice.current.userInterfaceIdiom == .pad ? 180 : 260
    }

    private(set) var assetModel: AssetViewModel
    private(set) var address: String

    var presentation: ReceivePresentationType?
    var renderedImage: UIImage?

    private let wallet: Wallet
    private let service: any GemReceiveServiceProtocol
    private let generator = QRCodeGenerator()
    let networkAssetIds: [AssetId]

    private init(
        asset: Asset,
        associations: [AssetAssociation],
        wallet: Wallet,
        address: String,
        service: any GemReceiveServiceProtocol,
    ) {
        assetModel = AssetViewModel(asset: asset)
        self.wallet = wallet
        self.address = address
        self.service = service
        networkAssetIds = service.networkAssetIds(
            assetId: asset.id.identifier,
            associations: associations.map(\.assetId.identifier),
            wallet: wallet.json(),
        ).map { AssetId(core: $0) }
    }

    public convenience init(assetData: AssetData, wallet: Wallet, service: any GemReceiveServiceProtocol) {
        self.init(
            asset: assetData.asset,
            associations: assetData.associations,
            wallet: wallet,
            address: assetData.account.address,
            service: service,
        )
    }

    public convenience init(assetAddress: AssetAddress, wallet: Wallet, service: any GemReceiveServiceProtocol) {
        self.init(
            asset: assetAddress.asset,
            associations: [],
            wallet: wallet,
            address: assetAddress.address,
            service: service,
        )
    }

    var title: String {
        Localized.Receive.title("")
    }

    var addressShort: String {
        GemAddressService.shared.format(address: address, chain: assetModel.asset.chain)
    }

    var shareTitle: String {
        Localized.Common.share
    }

    var copyTitle: String {
        Localized.Common.copy
    }

    var warningMessage: String {
        [Localized.Receive.warning(assetModel.symbol.boldMarkdown(), assetModel.networkFullName.boldMarkdown()), memoWarningText]
            .compactMap(\.self)
            .joined(separator: " ")
    }

    private var memoWarningText: String? {
        switch service.memoWarning(chain: assetModel.asset.chain.rawValue) {
        case .destinationTag: Localized.Wallet.Receive.noDestinationTagRequired
        case .memo: Localized.Wallet.Receive.noMemoRequired
        case .notSupported: nil
        }
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(
            type: .address(assetModel.asset, address: addressShort),
            copyValue: address,
        )
    }

    var showNetworkSelector: Bool {
        networkAssetIds.count > 1
    }

    var networkSelectorModel: ReceiveNetworkSelectorViewModel {
        ReceiveNetworkSelectorViewModel(
            assetIds: networkAssetIds,
        )
    }

    var isPresentingSheet: ReceivePresentationType? {
        get {
            switch presentation {
            case .share, .networkSelector: presentation
            case .copy, nil: nil
            }
        }
        set {
            presentation = newValue
        }
    }

    var isPresentingCopyToast: Bool {
        get {
            if case .copy = presentation {
                return true
            }
            return false
        }
        set {
            presentation = newValue ? .copy : nil
        }
    }

    func activityItems(qrImage: UIImage?) -> [Any] {
        if let qrImage {
            return [qrImage, address]
        }
        return [address]
    }

    private func enableAsset() async {
        do {
            try await service.enableAsset(walletId: wallet.id.id, assetId: assetModel.asset.id.identifier)
        } catch {
            debugLog("ReceiveViewModel enableAsset error: \(error)")
        }
    }

    private func prefetchAssociations() async {
        do {
            _ = try await service.syncMissingAssets(assetIds: networkAssetIds.filter { $0 != assetModel.asset.id }.ids)
        } catch {
            debugLog("ReceiveViewModel prefetchAssociations error: \(error)")
        }
    }

    private func generateQRCode() async -> UIImage? {
        await generator.generate(
            from: address,
            size: CGSize(
                width: qrSize,
                height: qrSize,
            ),
            logo: UIImage.name("logo-dark"),
        )
    }
}

// MARK: - Actions

extension ReceiveViewModel {
    func onTaskOnce() {
        Task {
            await enableAsset()
            await prefetchAssociations()
        }
    }

    func onSelectNetwork() {
        presentation = .networkSelector
    }

    func onFinishNetworkSelection(_ items: [ReceiveNetworkItem]) {
        presentation = nil
        guard let assetId = items.first?.assetId, assetId != assetModel.asset.id else { return }

        Task {
            do {
                let asset = try await service.asset(assetId: assetId.identifier).map()
                let account = try wallet.account(for: asset.chain)
                assetModel = AssetViewModel(asset: asset)
                address = account.address
                renderedImage = await generateQRCode()
                await enableAsset()
            } catch {
                debugLog("ReceiveViewModel onFinishNetworkSelection error: \(error)")
            }
        }
    }

    func onShareSheet() {
        presentation = .share
    }

    func onCopyAddress() {
        presentation = .copy
    }

    func onLoadImage() async {
        renderedImage = await generateQRCode()
    }
}
