// Renders the DexHowTo HealthTask app icon to a 1024x1024 PNG.
// Usage: swift scripts/make_icon.swift <output.png>
import AppKit

let size: CGFloat = 1024
let output = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "icon_1024.png"

let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: Int(size), pixelsHigh: Int(size),
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
let ctx = NSGraphicsContext.current!.cgContext

// macOS icon grid: 824pt rounded square centered in 1024 canvas.
let tile = CGRect(x: 100, y: 100, width: 824, height: 824)
let tilePath = NSBezierPath(roundedRect: tile, xRadius: 185, yRadius: 185)

// Soft drop shadow under the tile.
ctx.saveGState()
ctx.setShadow(offset: CGSize(width: 0, height: -12), blur: 28,
              color: NSColor.black.withAlphaComponent(0.35).cgColor)
NSColor.white.setFill()
tilePath.fill()
ctx.restoreGState()

// Background gradient: warm coral to deep rose.
ctx.saveGState()
tilePath.addClip()
let gradient = NSGradient(colors: [
    NSColor(srgbRed: 1.00, green: 0.42, blue: 0.42, alpha: 1),
    NSColor(srgbRed: 0.91, green: 0.16, blue: 0.40, alpha: 1),
    NSColor(srgbRed: 0.62, green: 0.10, blue: 0.42, alpha: 1),
], atLocations: [0, 0.55, 1], colorSpace: .sRGB)!
gradient.draw(in: tile, angle: -60)

// Subtle top highlight.
let gloss = NSGradient(colors: [
    NSColor.white.withAlphaComponent(0.22), NSColor.white.withAlphaComponent(0),
])!
gloss.draw(in: CGRect(x: tile.minX, y: tile.midY, width: tile.width, height: tile.height / 2), angle: -90)

// Faint large heart watermark.
func heartPath(center: CGPoint, width w: CGFloat) -> NSBezierPath {
    let h = w * 0.92
    let p = NSBezierPath()
    let bottom = CGPoint(x: center.x, y: center.y - h / 2)
    let top = CGPoint(x: center.x, y: center.y + h * 0.22)
    p.move(to: bottom)
    p.curve(to: CGPoint(x: center.x - w / 2, y: center.y + h * 0.18),
            controlPoint1: CGPoint(x: center.x - w * 0.18, y: center.y - h * 0.22),
            controlPoint2: CGPoint(x: center.x - w / 2, y: center.y - h * 0.10))
    p.curve(to: top,
            controlPoint1: CGPoint(x: center.x - w / 2, y: center.y + h * 0.62),
            controlPoint2: CGPoint(x: center.x - w * 0.06, y: center.y + h * 0.60))
    p.curve(to: CGPoint(x: center.x + w / 2, y: center.y + h * 0.18),
            controlPoint1: CGPoint(x: center.x + w * 0.06, y: center.y + h * 0.60),
            controlPoint2: CGPoint(x: center.x + w / 2, y: center.y + h * 0.62))
    p.curve(to: bottom,
            controlPoint1: CGPoint(x: center.x + w / 2, y: center.y - h * 0.10),
            controlPoint2: CGPoint(x: center.x + w * 0.18, y: center.y - h * 0.22))
    p.close()
    return p
}
NSColor.white.withAlphaComponent(0.09).setFill()
heartPath(center: CGPoint(x: 512, y: 470), width: 760).fill()
ctx.restoreGState()

// "DH" monogram in rounded heavy type.
let baseFont = NSFont.systemFont(ofSize: 360, weight: .heavy)
let font = baseFont.fontDescriptor.withDesign(.rounded).flatMap { NSFont(descriptor: $0, size: 360) } ?? baseFont
let shadow = NSShadow()
shadow.shadowColor = NSColor.black.withAlphaComponent(0.25)
shadow.shadowOffset = NSSize(width: 0, height: -8)
shadow.shadowBlurRadius = 18
let attrs: [NSAttributedString.Key: Any] = [
    .font: font, .foregroundColor: NSColor.white, .shadow: shadow, .kern: -6,
]
let text = NSAttributedString(string: "DH", attributes: attrs)
let textSize = text.size()
text.draw(at: CGPoint(x: 512 - textSize.width / 2, y: 430))

// Heartbeat (ECG) line across the lower part, ending in a small heart.
let pulse = NSBezierPath()
pulse.lineWidth = 34
pulse.lineCapStyle = .round
pulse.lineJoinStyle = .round
let y: CGFloat = 300
pulse.move(to: CGPoint(x: 210, y: y))
pulse.line(to: CGPoint(x: 360, y: y))
pulse.line(to: CGPoint(x: 410, y: y + 70))
pulse.line(to: CGPoint(x: 470, y: y - 120))
pulse.line(to: CGPoint(x: 530, y: y + 130))
pulse.line(to: CGPoint(x: 580, y: y))
pulse.line(to: CGPoint(x: 660, y: y))
ctx.saveGState()
ctx.setShadow(offset: CGSize(width: 0, height: -6), blur: 14,
              color: NSColor.black.withAlphaComponent(0.22).cgColor)
NSColor.white.setStroke()
pulse.stroke()
NSColor.white.setFill()
heartPath(center: CGPoint(x: 748, y: y + 4), width: 150).fill()
ctx.restoreGState()

NSGraphicsContext.restoreGraphicsState()
let png = rep.representation(using: .png, properties: [:])!
try! png.write(to: URL(fileURLWithPath: output))
print("Wrote \(output)")
