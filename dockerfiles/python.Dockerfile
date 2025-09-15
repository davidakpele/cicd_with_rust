# Use a lightweight Python base image
FROM python:3.9-slim

# Set the working directory
WORKDIR /app

# Copy the requirements file and install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy the rest of the application code
COPY . .

# Expose the application port (e.g., for Flask or Django)
EXPOSE 8000

# Run the application
CMD ["python", "app.py"]