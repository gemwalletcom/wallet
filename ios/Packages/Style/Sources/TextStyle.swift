// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI

public struct TextStyle: Sendable {
    public let font: Font
    public let fontWeight: Font.Weight?
    public let color: Color
    public let background: Color

    public init(
        font: Font,
        color: Color,
        fontWeight: Font.Weight? = nil,
        background: Color = Colors.black,
    ) {
        self.font = font
        self.color = color
        self.fontWeight = fontWeight
        self.background = background
    }

    public func weight(_ weight: Font.Weight) -> TextStyle {
        TextStyle(
            font: font,
            color: color,
            fontWeight: weight,
            background: background,
        )
    }
}

// MARK: - TextStyle Static

public extension TextStyle {
    static let title = TextStyle(font: .title, color: Colors.black)
    static let title2 = TextStyle(font: .title2, color: Colors.black)
    static let title3 = TextStyle(font: .title3, color: Colors.black)
    static let headline = TextStyle(font: .headline, color: Colors.black)
    static let subHeadline = TextStyle(font: .subheadline, color: Colors.secondaryText)
    static let body = TextStyle(font: .body, color: Colors.black)
    static let bodySecondary = TextStyle(font: .body, color: Colors.secondaryText)
    static let callout = TextStyle(font: .callout, color: Colors.black)
    static let calloutSecondary = TextStyle(font: .callout, color: Colors.secondaryText)
    static let footnote = TextStyle(font: .footnote, color: Colors.secondaryText)
    static let caption = TextStyle(font: .caption, color: Colors.secondaryText)
    static let largeTitle = TextStyle(font: .largeTitle, color: Colors.black)
    static let boldTitle = TextStyle(font: .title, color: Colors.black, fontWeight: .bold)
    static let highlighted = TextStyle(font: .headline, color: .white, background: Colors.blue)
}

// MARK: - Modifier

struct TextStyleModifier: ViewModifier {
    let style: TextStyle

    func body(content: Content) -> some View {
        content
            .font(style.font)
            .fontWeight(style.fontWeight)
            .foregroundStyle(style.color)
    }
}

// MARK: -

public extension View {
    func textStyle(_ style: TextStyle) -> some View {
        modifier(TextStyleModifier(style: style))
    }
}

public extension Text {
    func textStyle(_ style: TextStyle) -> some View {
        modifier(TextStyleModifier(style: style))
    }
}

// MARK: - Previews

#Preview {
    VStack(spacing: 16) {
        Text("Title")
            .textStyle(.title)
        Text("Title 2")
            .textStyle(.title2)
        Text("Title 3")
            .textStyle(.title3)
        Text("Headline")
            .textStyle(.headline)
        Text("Subheadline")
            .textStyle(.subHeadline)
        Text("Body text that provides additional information.")
            .textStyle(.body)
        Text("Body Secondary")
            .textStyle(.bodySecondary)
        Text("Callout")
            .textStyle(.callout)
        Text("Callout Secondary")
            .textStyle(.calloutSecondary)
        Text("Footnote")
            .textStyle(.footnote)
        Text("Caption")
            .textStyle(.caption)
        Text("Large Title")
            .textStyle(.largeTitle)
        Text("Bold Title")
            .textStyle(.boldTitle)
        Text("Highlighted Text")
            .textStyle(.highlighted)
    }
    .padding()
}
