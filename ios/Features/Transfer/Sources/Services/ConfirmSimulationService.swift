// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import func Gemstone.simulationHeader
import func Gemstone.simulationPayloadFields
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store

public struct ConfirmSimulationService: Sendable {
    private let nameService: any GemNameServiceProtocol
    private let assetsService: any GemAssetsServiceProtocol
    private let assetStore: AssetStore

    public init(
        nameService: any GemNameServiceProtocol,
        assetsService: any GemAssetsServiceProtocol,
        assetStore: AssetStore,
    ) {
        self.nameService = nameService
        self.assetsService = assetsService
        self.assetStore = assetStore
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
        async let names = nameService.addressNames(requests: addressRequests)
        do {
            try await assetsService.syncMissingAssets(for: simulation?.simulationAssetIds ?? [])
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
        let showsHeader = shouldHideValueField(for: transferType, simulation: simulation)
        guard let fields = try? simulationPayloadFields(payload: payload.map { try $0.json() }, showsHeader: showsHeader) else {
            return payload
        }
        return fields.compactMap { try? SimulationPayloadField($0) }
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
        let assets = (try? assetStore.getAssets(for: simulation.simulationAssetIds.ids)) ?? []
        return Dictionary(uniqueKeysWithValues: assets.map { ($0.id, $0) })
    }

    func shouldHideValueField(for transferType: TransferDataType, simulation: SimulationResult?) -> Bool {
        if approvalHeaderData(for: transferType) != nil {
            return true
        }

        return simulationHeaderValue(simulation) != nil
    }

    func simulationHeaderValue(_ simulation: SimulationResult?) -> (assetId: AssetId, value: ApprovalValue)? {
        guard let header = try? simulationHeader(simulation: simulation?.json()).flatMap({ try SimulationHeader($0) }),
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
