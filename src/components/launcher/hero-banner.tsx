import { cn } from "@/lib/utils"

interface HeroBannerProps {
  className?: string
}

export function HeroBanner({ className }: HeroBannerProps) {
  return (
    <div
      className={cn(
        "relative h-[400px] w-full overflow-hidden",
        className
      )}
    >
      {/* Banner Image */}
      <div className="absolute inset-0">
        <img
          src="/banner.png"
          alt="Pokemon Infinite Fusion"
          className="h-full w-full object-cover"
          onError={(e: React.SyntheticEvent<HTMLImageElement>) => {
            // Fallback gradient if image doesn't exist
            e.currentTarget.style.display = "none"
            if (e.currentTarget.parentElement) {
              e.currentTarget.parentElement.style.background =
                "linear-gradient(135deg, hsl(var(--pokemon-blue)) 0%, hsl(var(--fusion-purple)) 50%, hsl(var(--fusion-pink)) 100%)"
            }
          }}
        />
      </div>

      {/* Gradient Overlay for smooth transition */}
      <div className="absolute inset-0 bg-gradient-to-b from-transparent via-transparent to-background" />
    </div>
  )
}