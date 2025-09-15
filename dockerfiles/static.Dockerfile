# Use a lightweight Nginx image
FROM nginx:alpine

# Copy your static files to the Nginx web root directory
COPY . /usr/share/nginx/html

# Expose the standard HTTP port
EXPOSE 80

# The default Nginx command will run, serving the static content
CMD ["nginx", "-g", "daemon off;"]