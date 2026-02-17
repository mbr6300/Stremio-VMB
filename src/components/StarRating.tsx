interface StarRatingProps {
  value: number;
  size?: "small" | "large";
  onChange?: (rating: number) => void;
}

export default function StarRating({ value, size = "small", onChange }: StarRatingProps) {
  const sizeClass = size === "large" ? "star-rating-large" : "star-rating-small";
  return (
    <div className={`star-rating ${sizeClass}`}>
      {[1, 2, 3, 4, 5].map((r) => (
        <button
          key={r}
          type="button"
          className={`star ${r <= value ? "filled" : ""}`}
          onClick={() => onChange?.(r)}
          disabled={!onChange}
        >
          ★
        </button>
      ))}
    </div>
  );
}
