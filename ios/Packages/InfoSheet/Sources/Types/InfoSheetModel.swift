// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemConfirmErrorInfo
import enum Gemstone.GemInfoAction
import struct Gemstone.GemInfoSheet
import enum Gemstone.GemInfoTopic
import Localization
import PrimitivesComponents
import Style
import SwiftUI

public typealias InfoSheetAction = @MainActor @Sendable () -> Void
public typealias InfoSheetActionHandler = @MainActor @Sendable (GemInfoAction) -> Void

public enum InfoSheetButton: Sendable {
    case url(URL, title: String? = nil)
    case action(title: String, action: InfoSheetAction)

    var title: String {
        switch self {
        case let .url(_, title): title ?? Localized.Common.learnMore
        case let .action(title, _): title
        }
    }
}

public struct InfoSheetModel: Sendable {
    public let title: String
    public let description: String
    public let image: InfoSheetImage?
    public let button: InfoSheetButton?
    public let secondaryButtons: [InfoSheetButton]
    public let titleStyle: TextStyle
    public let descriptionStyle: TextStyle

    public var buttonTitle: String {
        button?.title ?? Localized.Common.learnMore
    }

    public init(
        title: String,
        description: String,
        image: InfoSheetImage? = nil,
        button: InfoSheetButton? = nil,
        secondaryButtons: [InfoSheetButton] = [],
        titleStyle: TextStyle = .boldTitle,
        descriptionStyle: TextStyle = .bodySecondary,
    ) {
        self.title = title
        self.description = description
        self.image = image
        self.button = button
        self.secondaryButtons = secondaryButtons
        self.titleStyle = titleStyle
        self.descriptionStyle = descriptionStyle
    }

    var shouldShowButton: Bool {
        button != nil || !secondaryButtons.isEmpty
    }
}

extension InfoSheetModel {
    init(sheet: GemInfoSheet, onAction: InfoSheetActionHandler?) {
        self.init(
            title: sheet.title.text,
            description: sheet.description.text,
            image: sheet.image.sheetImage,
            button: sheet.button(handlesActions: onAction != nil).flatMap { action in
                switch action {
                case let .learnMore(url): URL(string: url).map { .url($0) }
                case .buy, .acquire, .continue: onAction.map { onAction in .action(title: action.title, action: { onAction(action) }) }
                }
            },
        )
    }
}

extension GemInfoSheet: @retroactive Identifiable {
    public var id: Self { self }
}

public extension GemInfoTopic {
    var infoSheet: GemInfoSheet {
        sheet(platform: .ios)
    }
}

public extension GemConfirmErrorInfo {
    var infoSheet: GemInfoSheet {
        sheet(platform: .ios)
    }
}
