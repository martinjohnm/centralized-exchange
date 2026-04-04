# --- Stage 1: Build ---
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build

# --- Stage 2: Runtime (Production Grade) ---
FROM nginx:alpine
# Copy the build output from the builder stage to Nginx
COPY --from=builder /app/dist /usr/share/nginx/html
# Expose port 80 for the browser
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]