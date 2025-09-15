# Stage 1: Build the application
# Use a newer Node.js version that meets the requirements (e.g., node:20-alpine)
FROM node:20-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm install

COPY . .

# Run the build command
RUN npm run build

# Stage 2: Serve the application with Nginx
FROM nginx:alpine

# Copy the build output from the builder stage into Nginx's web root
COPY --from=builder /app/dist /usr/share/nginx/html

# Expose the standard HTTP port
EXPOSE 80

# The default Nginx command will serve the content
CMD ["nginx", "-g", "daemon off;"]