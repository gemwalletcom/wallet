// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import BigInt
import Primitives
import PrimitivesComponents

public struct ConfirmSimulationService: Sendable {
    private let addressNameService: AddressNameService
    private let assetsService: AssetsService

    public init(
        addressNameService: AddressNameService,
        assetsService: AssetsService,
    ) {
        self.addressNameService = addressNameService
        self.assetsService = assetsService
    }

    func makeState(data: TransferData, simulation: SimulationResult?) -> ConfirmSimulationState {
        let assets = simulationAssets(simulation)
        return ConfirmSimulationState(
            result: simulation,
            warnings: simulation?.warnings ?? [],
            payload: payloadModel(data: data, simulation: simulation),
            headerData: cachedHeaderData(data: data, simulation: simulation, assets: assets),
            balanceChanges: balanceChanges(simulation: simulation, assets: assets),
        )
    }

    func updateState(data: TransferData, simulation: SimulationResult?) async -> ConfirmSimulationState {
        var payload = payloadModel(data: data, simulation: simulation)
        let addressRequests = payload.addressRequests
        async let names = addressNameService.getAddressNames(requests: addressRequests)
        do {
            try await assetsService.prefetchAssets(assetIds: simulation?.simulationAssetIds ?? [])
        } catch {
            debugLog("simulation asset prefetch error: \(error)")
        }
        payload.addressNames = await (try? names) ?? [:]

        let assets = simulationAssets(simulation)
        return ConfirmSimulationState(
            result: simulation,
            warnings: simulation?.warnings ?? [],
            payload: payload,
            headerData: cachedHeaderData(data: data, simulation: simulation, assets: assets),
            balanceChanges: balanceChanges(simulation: simulation, assets: assets),
        )
    }
}

private extension ConfirmSimulationService {
    func payloadModel(data: TransferData, simulation: SimulationResult?) -> SimulationPayloadModel {
        let fields = payloadFields(for: data.type, simulation: simulation)
        return SimulationPayloadModel(
            chain: data.chain,
            primaryFields: fields.primaryFields,
            secondaryFields: fields.secondaryFields,
        )
    }

    func payloadFields(
        for transferType: TransferDataType,
        simulation: SimulationResult?,
    ) -> [SimulationPayloadField] {
        guard case .generic = transferType else {
            return []
        }

        let payload = simulation?.payload ?? []
        guard shouldHideValueField(for: transferType, simulation: simulation) else {
            return payload
        }

        return payload.filter { $0.kind != .value }
    }

    func approvalHeaderData(for transferType: TransferDataType) -> AssetValueHeaderData? {
        guard case let .tokenApprove(asset, approval) = transferType,
              let value = approval.approvalValue
        else {
            return nil
        }

        return AssetValueHeaderData(asset: asset, value: value)
    }

    func cachedHeaderData(
        data: TransferData,
        simulation: SimulationResult?,
        assets: [AssetId: Asset],
    ) -> AssetValueHeaderData? {
        if let headerData = approvalHeaderData(for: data.type) {
            return headerData
        }

        guard case .generic = data.type,
              let headerValue = simulationHeaderValue(simulation)
        else {
            return nil
        }

        guard let asset = assets[headerValue.assetId] else {
            return nil
        }
        return AssetValueHeaderData(asset: asset, value: headerValue.value)
    }

    func balanceChanges(
        simulation: SimulationResult?,
        assets: [AssetId: Asset],
    ) -> [SimulationAssetChange] {
        (simulation?.balanceChanges ?? []).compactMap { change in
            guard let value = BigInt(change.value, radix: 10),
                  value != .zero,
                  let asset = assets[change.assetId]
            else {
                return nil
            }
            return SimulationAssetChange(
                asset: asset,
                value: value,
            )
        }
    }

    func simulationAssets(_ simulation: SimulationResult?) -> [AssetId: Asset] {
        guard let simulation else {
            return [:]
        }
        let assets = (try? assetsService.getAssets(for: simulation.simulationAssetIds)) ?? []
        return Dictionary(uniqueKeysWithValues: assets.map { ($0.id, $0) })
    }

    func shouldHideValueField(for transferType: TransferDataType, simulation: SimulationResult?) -> Bool {
        if approvalHeaderData(for: transferType) != nil {
            return true
        }

        return simulationHeaderValue(simulation) != nil
    }

    func simulationHeaderValue(_ simulation: SimulationResult?) -> (assetId: AssetId, value: ApprovalValue)? {
        guard let header = simulation?.header,
              let value = header.approvalValue
        else {
            return nil
        }
        return (header.assetId, value)
    }
}

private extension SimulationResult {
    var simulationAssetIds: [AssetId] {
        balanceChanges.map(\.assetId) + [header?.assetId].compactMap(\.self)
    }
}
